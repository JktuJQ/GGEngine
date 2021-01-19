#include "../headers/errors.h"



// GGEngineError
gg::GGEngineError::GGEngineError(const std::string& message)
    : std::runtime_error(message) {

}

gg::GGEngineError::~GGEngineError() = default;
