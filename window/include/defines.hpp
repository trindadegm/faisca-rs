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

    enum AppMessageType {
        APPMSG_SET_WINDOW_SIZE = 1,
        APPMSG_SET_FULLSCREEN,
        APPMSG_SET_BORDERLESS,
        APPMSG_SET_WINDOW_TITLE,
        APPMSG_CREATE_VULKAN_SURFACE,
        APPMSG_QUERY_VIEWPORT_EXTENT,
        APPMSG_SET_MSG_BACKCHANNEL,
    };

    struct AppMessage {
        uint32_t type;
        union {
            struct {
                uint32_t width;
                uint32_t height;
            } windowSize;
            uint8_t fullscreen;
            uint8_t borderless;
            const char *windowTitle;
            struct {
                uint64_t instance_handle;
                struct {
                    void *out;
                    const void *barrier;
                } *responseBinding;
            } windowSurfaceCreateInfo;
            struct {
                void *out;
                const void *barrier;
            } *queryResponseBinding;
            void *msgBackchannel;
        };
    };

    enum WindowEventType {
        WINEVT_QUIT = 1,
    };

    struct WindowEvent {
        uint32_t type;
        union {
            uint32_t keyDown;
        };
    };

    enum WindowMessageType {
        WINMSG_VULKAN_INSTANCE_REQUIRED_EXTENSIONS = 1,
        WINMSG_RESPONSE_NOTIFY,
        WINMSG_WINDOW_EVENT,
    };

    struct WindowMessage {
        uint32_t type;
        union {
            struct {
                const char *const *names;
                size_t count;
            } vk_instance_required_ext;
            void *responseNotifyBinding;
            struct {
                void *msgBackchannel;
                const WindowEvent *windowEvent;
            } windowEvent;
        };
    };

    struct Extent2D {
        uint32_t width;
        uint32_t height;
    };

    typedef void* WindowInstance;

    typedef uint32_t (ECABI *FnMessageWindow)(WindowInstance, const AppMessage*);

    typedef void (ECABI *FnRunApp)(WindowInstance, FnMessageWindow);
    typedef uint32_t (ECABI *FnMessageApp)(WindowInstance, const WindowMessage*);
}

#endif // FAISCA_DEFINES_HPP_