#ifndef FAISCA_DEFINES_HPP_
#define FAISCA_DEFINES_HPP_

#ifdef _WIN32
#include <windows.h>
#define ECABI __cdecl
#else
#error "Not yet implemented"
#endif

#include <cstdint>

namespace faisca {
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

    struct WindowMessage {
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

    enum AppMessageType {
        VULKAN_INSTANCE_REQUIRED_EXTENSIONS = 1,
    };

    struct AppMessage {
        uint32_t type;
        union {
            struct {
                const char *const *names;
                size_t count;
            } vk_instance_required_ext;
        };
    };

    typedef uint32_t (ECABI *FnMessageWindow)(const WindowMessage*);

    typedef void (ECABI *FnRunApp)(FnMessageWindow);
    typedef uint32_t (ECABI *FnMessageApp)(const AppMessage*);
}

#endif // FAISCA_DEFINES_HPP_