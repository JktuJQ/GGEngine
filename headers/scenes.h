#ifndef GGENGINE_SCENES_H
#define GGENGINE_SCENES_H

// Includes
#include "declarations.h"



namespace gg {

    class Scene final {
        // Scene class which handles all gameobjects
    public:
        const std::string name;


        Scene(std::string name);

        unsigned int addGameObject(GameObject *gameobject); // Add gameobject to scene
        void removeGameObject(unsigned int index); // Remove gameobject from scene
        GameObject *&getGameObject(unsigned int index); // Return gameobject on index

        std::vector<GameObject*> &getGameObjects();

    private:
        std::vector<GameObject*> gameobjects;
    };



    class SceneManager final {
        // SceneManager handles Scene classes in
    public:
        SceneManager();

        void addScene(Scene *scene); // Adds scene to scene manager
        void removeScene(const std::string& scene_name); // Removes scene from scene manager
        Scene *&getScene(const std::string& scene_name); // Returns scene with the same name

        std::map<std::string, Scene*> &getScenes();

    private:
        std::map<std::string, Scene*> scenes;
    };

}



#endif
