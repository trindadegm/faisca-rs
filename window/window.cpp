#include <cstdlib>
#include <cstring>
#include <iostream>
#include <vector>
#include <SDL.h>
#include <thread>

#ifdef __WIN32__
#include <windows.h>
#define ECABI __cdecl
#endif

enum FullscreenType {
    FULLSCREEN_NONE = 0,
    FULLSCREEN_REAL = 1,
    FULLSCREEN_DESKTOP = 2,
};

enum RendererMessageType {
    SET_WINDOW_SIZE = 1,
    SET_FULLSCREEN,
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

typedef uint32_t (ECABI *FnMessageRenderer)(const RendererMessage*);
typedef void (ECABI *FnRunApp)(FnMessageRenderer);

static uint32_t gUserEventNum = 0;

extern "C" {
    uint32_t ECABI FaiscaMessageRenderer(const RendererMessage *msg) {
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

FnRunApp getFaiscaAppFn(const char* sharedObjectFilepath);

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
        std::cerr << "Failed to create SDL2 window: " << SDL_GetError() << std::endl;
        return 1;
    }

    uint32_t customEventType = SDL_RegisterEvents(1);
    if (customEventType == 0xFFFFFFFF) {
        std::cerr << "Failed to register user event: " << SDL_GetError() << std::endl;
        return 1;
    }
    gUserEventNum = customEventType;

    FnRunApp runApp = getFaiscaAppFn(sharedObjectFilepath);
    std::thread appFnThread(runApp, FaiscaMessageRenderer);

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

FnRunApp ECABI getFaiscaAppFn(const char *sharedObjectFilepath) {
    int requiredLength = MultiByteToWideChar(CP_UTF8, MB_PRECOMPOSED, sharedObjectFilepath, -1, NULL, 0);
    wchar_t *sharedObjectFilepathW = new wchar_t[requiredLength];
    MultiByteToWideChar(CP_UTF8, MB_PRECOMPOSED, sharedObjectFilepath, -1, sharedObjectFilepathW, requiredLength);

    HINSTANCE instance = LoadLibraryW(sharedObjectFilepathW);
    if (instance == NULL) {
        std::cerr << "Failed to load '" << sharedObjectFilepath << "'" <<std::endl;
        exit(1);
    }

    FnRunApp fnRunApp = reinterpret_cast<FnRunApp>(GetProcAddress(instance, "faisca_run_app"));
    if (fnRunApp == nullptr) {
        std::cerr << "Failed to load 'faisca_run_app' from DLL" << std::endl;
        exit(1);
    }

    return fnRunApp;
}