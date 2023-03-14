#include <chrono>
#include <condition_variable>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <mutex>
#include <SDL.h>
#include <SDL_vulkan.h>
#include <thread>
#include <vector>

#include <defines.hpp>
#include <dylib.hpp>

using namespace faisca;
using namespace std::chrono_literals;

static uint32_t gUserEventNum = 0;

static std::mutex gEventWaitMutex;
static std::condition_variable gEventWaitFence;
static bool gEventWaitFlag = false;

extern "C" {
    uint32_t ECABI FaiscaMessageWindow(WindowInstance win, const AppMessage *msg) {
        AppMessage *ourMessage = new AppMessage;
        *ourMessage = *msg;
        switch (msg->type) {
            case APPMSG_SET_WINDOW_TITLE: {
                // We must copy the pointed data so that we own it
                // The +1 is to account for the NULL byte
                size_t strLength = strnlen(msg->windowTitle, 255) + 1;
                char *ourString = new char[strLength];
                SDL_strlcpy(ourString, msg->windowTitle, strLength);
                // We now point to our allocated string
                ourMessage->windowTitle = ourString;
            } break;
            default:
                break;
        }

        SDL_Event e = {};
        e.type = SDL_USEREVENT;
        e.user.type = gUserEventNum;
        e.user.code = ourMessage->type;
        e.user.data1 = ourMessage;
        e.user.data2 = win;

        if (SDL_PushEvent(&e) != 0) {
            // If pushed an event, notify to wake the event thread earlier.
            {
                std::unique_lock<std::mutex> lk(gEventWaitMutex);
                gEventWaitFlag = true;
            }
            gEventWaitFence.notify_all();
            return 1;
        } else {
            return 0;
        }
    }
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        std::cerr << "Missing faisca game shared object argument" << std::endl;
        return 1;
    }
    const char* sharedObjectFilepath = argv[1];

    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_EVENTS) < 0) {
        std::cerr << "Failed to initialize SDL2: " << SDL_GetError() << std::endl;
        return 1;
    }

    SDL_Window* mainWindow = SDL_CreateWindow(
        "Faisca Window",
        SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED, 800, 450,
        SDL_WINDOW_SHOWN | SDL_WINDOW_VULKAN
    );

    if (mainWindow == nullptr) {
        std::cerr << "Failed to create SDL window: " << SDL_GetError() << std::endl;
        return 1;
    }

    uint32_t customEventType = SDL_RegisterEvents(1);
    if (customEventType == 0xFFFFFFFF) {
        std::cerr << "Failed to register user event: " << SDL_GetError() << std::endl;
        return 1;
    }
    gUserEventNum = customEventType;

    DyLib appLib(sharedObjectFilepath);
    FnRunApp runApp = reinterpret_cast<FnRunApp>(appLib.getProcAddr("faisca_run_app"));
    FnMessageApp messageApp = reinterpret_cast<FnMessageApp>(appLib.getProcAddr("faisca_message_app"));

    unsigned int numExtensions = 0;
    if (!SDL_Vulkan_GetInstanceExtensions(mainWindow, &numExtensions, nullptr)) {
        std::cerr << "Failed to query SDL instance extensions: " << SDL_GetError() << std::endl;
        return 1;
    }
    const char **requiredExtensions = new const char*[numExtensions];
    SDL_Vulkan_GetInstanceExtensions(mainWindow, &numExtensions, requiredExtensions);

    WindowMessage requiredExtensionMsg = {};
    requiredExtensionMsg.type = WINMSG_VULKAN_INSTANCE_REQUIRED_EXTENSIONS;
    requiredExtensionMsg.vk_instance_required_ext.names = requiredExtensions;
    requiredExtensionMsg.vk_instance_required_ext.count = numExtensions;
    messageApp(mainWindow, &requiredExtensionMsg);

    std::thread appFnThread(runApp, mainWindow, FaiscaMessageWindow);

    void *backChannel= nullptr;

    SDL_Event e;
    bool running = true;
    while (running) {
        // int res = SDL_WaitEvent(&e);
        if (SDL_PollEvent(&e) != 0) {
            switch (e.type) {
                case SDL_QUIT: {
                    running = false;
                    WindowEvent windowEvent {};
                    windowEvent.type = WINEVT_QUIT;

                    WindowMessage windowMessage {};
                    windowMessage.type = WINMSG_WINDOW_EVENT;
                    windowMessage.windowEvent.msgBackchannel = backChannel;
                    windowMessage.windowEvent.windowEvent = &windowEvent;

                    messageApp(mainWindow, &windowMessage);
                } break;
                case SDL_WINDOWEVENT_RESIZED: {
                    WindowEvent windowEvent = WindowEvent {};
                    windowEvent.type = WINEVT_WINDOW_RESIZE;
                    windowEvent.windowResize.w = e.window.data1;
                    windowEvent.windowResize.h = e.window.data2;

                    WindowMessage windowMessage {};
                    windowMessage.type = WINMSG_WINDOW_EVENT;
                    windowMessage.windowEvent.msgBackchannel = backChannel;
                    windowMessage.windowEvent.windowEvent = &windowEvent;

                    SDL_Window *eWindow = SDL_GetWindowFromID(e.window.windowID);

                    messageApp(eWindow, &windowMessage);
                } break;
                case SDL_USEREVENT: {
                    const AppMessage *msg = static_cast<const AppMessage*>(e.user.data1);
                    SDL_Window *msgWindow = static_cast<SDL_Window*>(e.user.data2);
                    switch (msg->type) {
                        case APPMSG_SET_WINDOW_SIZE:
                            SDL_SetWindowSize(msgWindow, msg->windowSize.width, msg->windowSize.height);
                            break;
                        case APPMSG_SET_FULLSCREEN:
                            SDL_SetWindowFullscreen(
                                msgWindow,
                                (msg->fullscreen == FULLSCREEN_NONE ?
                                    0 : (msg->fullscreen == FULLSCREEN_REAL ?
                                        SDL_WINDOW_FULLSCREEN : SDL_WINDOW_FULLSCREEN_DESKTOP
                                    )
                                )
                            );
                            break;
                        case APPMSG_SET_BORDERLESS:
                            SDL_SetWindowBordered(msgWindow, msg->borderless == 1 ? SDL_FALSE : SDL_TRUE);
                            break;
                        case APPMSG_SET_WINDOW_TITLE:
                            SDL_SetWindowTitle(msgWindow, msg->windowTitle);
                            delete[] msg->windowTitle;
                            break;
                        case APPMSG_CREATE_VULKAN_SURFACE: {
                            SDL_Vulkan_CreateSurface(
                                msgWindow,
                                (VkInstance) msg->windowSurfaceCreateInfo.instance_handle,
                                (VkSurfaceKHR*) msg->windowSurfaceCreateInfo.responseBinding->out
                            );
                            WindowMessage message = {};
                            message.type = WINMSG_RESPONSE_NOTIFY;
                            message.responseNotifyBinding = msg->windowSurfaceCreateInfo.responseBinding;
                            messageApp(msgWindow, &message);
                        } break;
                        case APPMSG_QUERY_VIEWPORT_EXTENT: {
                            Extent2D *vkExtent2D = (Extent2D*) msg->queryResponseBinding->out;
                            int width, height;
                            SDL_GetWindowSizeInPixels(msgWindow, &width, &height);
                            vkExtent2D->width = width;
                            vkExtent2D->height = height;

                            WindowMessage message = {};
                            message.type = WINMSG_RESPONSE_NOTIFY;
                            message.responseNotifyBinding = msg->queryResponseBinding;
                            messageApp(msgWindow, &message);
                        } break;
                        case APPMSG_SET_MSG_BACKCHANNEL:
                            backChannel = msg->msgBackchannel;
                            break;
                        default:
                            break;
                    }
                    delete msg;
                } break;
            }
        } // End of SDL_PollEvent

        {
            std::unique_lock<std::mutex> lk(gEventWaitMutex);
            gEventWaitFence.wait_for(lk, 8ms, []{return gEventWaitFlag;});
            gEventWaitFlag = false;
        }
    }

    appFnThread.join();

    SDL_DestroyWindow(mainWindow);
    SDL_Quit();

    return 0;
}
