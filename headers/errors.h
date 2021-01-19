#ifndef GGENGINE_ERRORS_H
#define GGENGINE_ERRORS_H

// Includes
#include "declarations.h"



namespace gg {

    class GGEngineError : public std::runtime_error {
    public:
        GGEngineError(const std::string& message);
        ~GGEngineError();
    };

}



#endif
