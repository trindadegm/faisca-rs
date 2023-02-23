#ifndef FAISCA_DYLIB_LOAD_HPP_
#define FAISCA_DYLIB_LOAD_HPP_

#include <defines.hpp>

namespace faisca {
    #ifdef _WIN32
    typedef HINSTANCE LibHandle;
    typedef FARPROC FnAddr;
    #else
    #error "Not yet implemented"
    #endif

    class DyLib final {
    private:
        LibHandle _handle;
    public:
        DyLib(const char *name);
        ~DyLib(void);

        FnAddr getProcAddr(const char *procName);
    };
}

#endif // DYLIB_LOAD_HPP_
