#include "../headers/events.h"



// Event
gg::Event::Event(std::string event_name)
    : event_name(std::move(event_name)) {

}

gg::Event::~Event() = default;


unsigned int gg::Event::addSlot(gg::Slot *function) {
    slots.push_back(function);
    return slots.size() - 1;
}

void gg::Event::removeSlot(unsigned int index) {
    slots.erase(slots.begin() + index);
}

void gg::Event::signal(gg::GameObject *self, gg::GameObject *sender) {
    for (const auto &slot : slots){
        slot(self, sender);
    }
}


std::vector<gg::Slot *> &gg::Event::getSlots() {
    return slots;
}



// EventHandler
gg::EventHandler::EventHandler(gg::GameObject *binded_gameobject)
    : events(), binded_gameobject(binded_gameobject) {

}


void gg::EventHandler::addEvent(gg::Event *event, bool is_default) {
    for (const auto &other_event : events){
        if (std::get<0>(other_event.first) == event->event_name){
            throw GGEngineError("This event name was already used");
        }
    }
    std::tuple<std::string, bool> key = {event->event_name, is_default};
    events.insert(std::pair<std::tuple<std::string, bool>, Event*>{key, event});
}

void gg::EventHandler::removeEvent(const std::string &event_name) {
    for (const auto &event : events){
        if (std::get<0>(event.first) == event_name){
            if (std::get<1>(event.first)){
                throw GGEngineError("This event marked as default, it can't be removed");
            }
            events.erase(event.first);
            break;
        }
    }
    throw GGEngineError("There is no event with that event_name");
}

gg::Event *gg::EventHandler::getEvent(const std::string &event_name) {
    for (const auto &event : events){
        if (std::get<0>(event.first) == event_name){
            return event.second;
        }
    }
    throw GGEngineError("There is no event with that event_name");
}

void gg::EventHandler::invokeEvent(const std::string &event_name, gg::GameObject *sender) {
    getEvent(event_name)->signal(binded_gameobject, sender);
}

std::map<std::tuple<std::string, bool>, gg::Event*> &gg::EventHandler::getEvents() {
    return events;
}
