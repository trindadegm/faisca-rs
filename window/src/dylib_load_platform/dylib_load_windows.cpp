#include <dylib.hpp>

namespace faisca {
    DyLib::DyLib(const char *name) {
        int requiredLength = MultiByteToWideChar(CP_UTF8, MB_PRECOMPOSED, name, -1, NULL, 0);

        wchar_t *nameW = new wchar_t[requiredLength];

        MultiByteToWideChar(CP_UTF8, MB_PRECOMPOSED, name, -1, nameW, requiredLength);
        _handle = LoadLibraryW(nameW);

        delete[] nameW;
    }

    DyLib::~DyLib(void) {
        FreeLibrary(_handle);
    }

    FnAddr DyLib::getProcAddr(const char *procName) {
        return GetProcAddress(_handle, procName);
    }
}