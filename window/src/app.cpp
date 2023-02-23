#include <app.hpp>

namespace faisca {
    App::App(const char *name)
    : _dylib = DyLib(name) {
        _runApp = _dylib.getProcAddr("faisca_run_app");
        _messageApp = _dylib.getProcAddr("faisca_message_app");
    }

    App::~App() {
    }
}