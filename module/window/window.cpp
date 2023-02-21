#include <cstdlib>
#include <cstring>
#include <iostream>
#include <vector>
#include <SDL.h>
#include <thread>

#ifdef __WIN32__
#include <windows.h>
typedef void (__cdecl *FnRunApp)(void);
#endif

enum RendererMessageType {
    SET_WINDOW_SIZE = 1,
    SET_FULLSCREEN,
    SET_FULLSCREEN_BORDERLESS_WINDOW,
    SET_BORDERLESS,
    SET_WINDOW_TITLE,
};

struct RendererMessage {
    uint32_t type;
    union {
        struct {
            uint32_t width;
            uint32_t height;
        } windowSize;
        uint8_t fullscreen;
        uint8_t borderless;
        const char *windowTitle;
    };
};

static uint32_t gUserEventNum = 0;

extern "C" {
    uint32_t FaiscaMessageRenderer(const RendererMessage *msg) {
        RendererMessage *ourMessage = new RendererMessage;
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

void startFaiscaAppThread(void);

int main(int argc, char *argv[]) {
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
        std::cerr << "Failed to create SDL2 window: " << SDL_GetError() << std::endl;
        return 1;
    }

    uint32_t customEventType = SDL_RegisterEvents(1);
    if (customEventType == 0xFFFFFFFF) {
        std::cerr << "Failed to register user event: " << SDL_GetError() << std::endl;
        return 1;
    }
    gUserEventNum = customEventType;

    startFaiscaAppThread();

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
                    const RendererMessage *msg = static_cast<const RendererMessage*>(e.user.data1);
                    switch (msg->type) {
                        case SET_WINDOW_SIZE:
                            SDL_SetWindowSize(window, msg->windowSize.width, msg->windowSize.height);
                            break;
                        case SET_FULLSCREEN:
                        case SET_FULLSCREEN_BORDERLESS_WINDOW:
                            SDL_SetWindowFullscreen(
                                window,
                                (msg->type == SET_FULLSCREEN ?
                                    SDL_WINDOW_FULLSCREEN : SDL_WINDOW_FULLSCREEN_DESKTOP
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

    SDL_DestroyWindow(window);
    SDL_Quit();

    return 0;
}

void startFaiscaAppThread(void) {
    HINSTANCE instance = LoadLibrary(TEXT("FaiscaApp.dll"));
    if (instance == NULL) {
        std::cerr << "Failed to load FaiscaApp.dll" << std::endl;
        exit(1);
    }

    FnRunApp fnRunApp = reinterpret_cast<FnRunApp>(GetProcAddress(instance, "faisca_run_app"));
    if (fnRunApp == nullptr) {
        std::cerr << "Failed to load 'faisca_run_app' from DLL" << std::endl;
    }

    std::thread t(fnRunApp);
}