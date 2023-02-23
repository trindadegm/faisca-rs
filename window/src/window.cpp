#include <cstdlib>
#include <cstring>
#include <iostream>
#include <vector>
#include <SDL.h>
#include <SDL_vulkan.h>
#include <thread>

#include <defines.hpp>
#include <dylib.hpp>

using namespace faisca;

static uint32_t gUserEventNum = 0;

extern "C" {
    uint32_t ECABI FaiscaMessageWindow(WindowInstance, const AppMessage *msg) {
        AppMessage *ourMessage = new AppMessage;
        *ourMessage = *msg;
        if (msg->type == SET_WINDOW_TITLE) {
            // We must copy the pointed data so that we own it
            // The +1 is to account for the NULL byte
            size_t strLength = strnlen(msg->windowTitle, 255) + 1;
            char *ourString = new char[strLength];
            SDL_strlcpy(ourString, msg->windowTitle, strLength);
            // We now point to our allocated string
            ourMessage->windowTitle = ourString;
        }
        SDL_Event e = {};
        e.type = SDL_USEREVENT;
        e.user.type = gUserEventNum;
        e.user.code = ourMessage->type;
        e.user.data1 = ourMessage;
        if (SDL_PushEvent(&e) != 0) {
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

    SDL_Window* window = SDL_CreateWindow(
        "Faisca Window",
        SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED, 800, 450,
        SDL_WINDOW_SHOWN | SDL_WINDOW_VULKAN
    );

    if (window == nullptr) {
        std::cerr << "Failed to create SDL window: " << SDL_GetError() << std::endl;
        return 1;
    }

    unsigned int numExtensions = 0;
    if (!SDL_Vulkan_GetInstanceExtensions(window, &numExtensions, nullptr)) {
        std::cerr << "Failed to query SDL instance extensions: " << SDL_GetError() << std::endl;
        return 1;
    }
    const char **requiredExtensions = new const char*[numExtensions];
    SDL_Vulkan_GetInstanceExtensions(window, &numExtensions, requiredExtensions);

    uint32_t customEventType = SDL_RegisterEvents(1);
    if (customEventType == 0xFFFFFFFF) {
        std::cerr << "Failed to register user event: " << SDL_GetError() << std::endl;
        return 1;
    }
    gUserEventNum = customEventType;

    DyLib appLib(sharedObjectFilepath);
    FnRunApp runApp = reinterpret_cast<FnRunApp>(appLib.getProcAddr("faisca_run_app"));
    FnMessageApp messageApp = reinterpret_cast<FnMessageApp>(appLib.getProcAddr("faisca_message_app"));
    std::thread appFnThread(runApp, window, FaiscaMessageWindow);

    SDL_Event e;
    bool running = true;
    while (running) {
        int res = SDL_WaitEvent(&e);
        if (res) {
            switch (e.type) {
                case SDL_QUIT:
                    running = false;
                    break;
                case SDL_USEREVENT: {
                    const AppMessage *msg = static_cast<const AppMessage*>(e.user.data1);
                    switch (msg->type) {
                        case SET_WINDOW_SIZE:
                            SDL_SetWindowSize(window, msg->windowSize.width, msg->windowSize.height);
                            break;
                        case SET_FULLSCREEN:
                            SDL_SetWindowFullscreen(
                                window,
                                (msg->fullscreen == FULLSCREEN_NONE ?
                                    0 : (msg->fullscreen == FULLSCREEN_REAL ?
                                        SDL_WINDOW_FULLSCREEN : SDL_WINDOW_FULLSCREEN_DESKTOP
                                    )
                                )
                            );
                            break;
                        case SET_BORDERLESS:
                            SDL_SetWindowBordered(window, msg->borderless == 1 ? SDL_FALSE : SDL_TRUE);
                            break;
                        case SET_WINDOW_TITLE:
                            SDL_SetWindowTitle(window, msg->windowTitle);
                            delete[] msg->windowTitle;
                            break;
                        default:
                            break;
                    }
                    delete msg;
                } break;
            }
        } else {
            std::cerr << "An error ocurred while waiting for an event: " << SDL_GetError() << std::endl;
        }
    }

    appFnThread.join();

    SDL_DestroyWindow(window);
    SDL_Quit();

    return 0;
}
