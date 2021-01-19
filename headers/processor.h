#ifndef GGENGINE_PROCESSOR_H
#define GGENGINE_PROCESSOR_H

// Includes
#include "declarations.h"
#include "components.h"
#include "gameobjects.h"
#include "scenes.h"



namespace gg {

    class Processor {
        // Processes scene and invokes events. Can be inherited, for handling custom components logic
    public:
        virtual void process(Scene *scene); // Processes scene
    };

}



#endif
