#include <dylib.hpp>
#include <iostream>

namespace faisca {
    DyLib::DyLib(const char *name) {
        int requiredLength = MultiByteToWideChar(CP_UTF8, MB_PRECOMPOSED, name, -1, NULL, 0);

        wchar_t *nameW = new wchar_t[requiredLength];

        MultiByteToWideChar(CP_UTF8, MB_PRECOMPOSED, name, -1, nameW, requiredLength);
        _handle = LoadLibraryW(nameW);
        if (_handle == NULL) {
            auto eCode = GetLastError();
            std::cerr << "Loading library failed with code: " << eCode << std::endl;
            throw std::runtime_error("Could not load library");
        }

        delete[] nameW;
    }

    DyLib::~DyLib(void) {
        FreeLibrary(_handle);
    }

    FnAddr DyLib::getProcAddr(const char *procName) {
        return GetProcAddress(_handle, procName);
    }
}