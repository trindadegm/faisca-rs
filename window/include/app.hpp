#ifndef FAISCA_FAISCA_APP_HPP_
#define FAISCA_FAISCA_APP_HPP_

#include <defines.hpp>
#include <dylib.hpp>

namespace faisca {
    class App final {
    private:
        DyLib _dylib;

        FnRunApp _runApp;
        FnMessageApp _messageApp;
    public:
        App(const char *name);
        ~App()

        void runApp(FnMessageWindow messageWindow) {
            _runApp(messageWindow);
        }

        uint32_t messageApp(AppMessage *msg) {
            return _messageApp(msg);
        }
    };
}

#endif // FAISCA_FAISCA_APP_HPP_