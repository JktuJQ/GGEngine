#include "../headers/gameobjects.h"

#include <utility>



// GameObject
gg::GameObject::GameObject(std::string name, std::string tag)
    : name(std::move(name)), tag(std::move(tag)), event_handler(this) {
    // Events
    event_handler.addEvent(new Event("on_collision"), true);
}

gg::GameObject::~GameObject() = default;


void gg::GameObject::addComponent(gg::Component *component) {
    components.insert(std::pair<std::string, Component*>{component->name, component});
}

void gg::GameObject::removeComponent(const std::string& component_name) {
    components.erase(component_name);
}

gg::Component *&gg::GameObject::getComponent(const std::string &component_name) {
    return components.at(component_name);
}


std::map<std::string, gg::Component*> &gg::GameObject::getComponents() {
    return components;
}

gg::EventHandler &gg::GameObject::getEventHandler() {
    return event_handler;
}
