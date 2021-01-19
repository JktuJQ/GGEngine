#ifndef GGENGINE_EVENTS_H
#define GGENGINE_EVENTS_H

// Includes
#include "declarations.h"
#include "errors.h"



namespace gg {

    typedef void Slot(GameObject *self, GameObject *sender);



    class Event {
        // Base event class
    public:
        const std::string event_name;


        Event(std::string event_name);
        virtual ~Event();

        unsigned int addSlot(Slot *function); // Adds slot to event, returns index of slot
        void removeSlot(unsigned int index); // Removes slot from event
        void signal(GameObject *self, GameObject *sender); // Invokes all event slot

        std::vector<Slot*> &getSlots();

    private:
        std::vector<Slot*> slots{};
    };



    class EventHandler final {
        // EventHandler which is used to handle all events and call them
    public:
        EventHandler(GameObject *binded_gameobject);

        void addEvent(Event *event, bool is_default = false); // Adds event to event handler
        void removeEvent(const std::string &event_name); // Removes event from event handler
        Event *getEvent(const std::string &event_name); // Returns pointer to event in event handler
        void invokeEvent(const std::string &event_name, GameObject *sender); // Calls event with the same name

        std::map<std::tuple<std::string, bool>, Event*> &getEvents();

    private:
        GameObject *binded_gameobject;
        std::map<std::tuple<std::string, bool>, Event*> events;
    };

}



#endif
