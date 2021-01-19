#ifndef GGENGINE_GAMEOBJECTS_H
#define GGENGINE_GAMEOBJECTS_H

// Includes
#include "declarations.h"
#include "events.h"
#include "components.h"



namespace gg {

    class GameObject {
        // Base GameObject class, container for components
    public:
        const std::string name, tag;


        GameObject(std::string name, std::string tag = DEFAULT_TAG);
        virtual ~GameObject();

        void addComponent(Component *component); // Adds component to gameobject
        void removeComponent(const std::string& component_name); // Remove component from gameobject
        Component *&getComponent(const std::string& component_name); // Returns component with choosed name

        std::map<std::string, Component*> &getComponents();
        EventHandler &getEventHandler();

    private:
        std::map<std::string, Component*> components;
        EventHandler event_handler;

    protected:
        inline static std::string DEFAULT_TAG = "gameobject";
    };

}



#endif
