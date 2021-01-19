#include "../headers/scenes.h"



// Scene
gg::Scene::Scene(std::string name)
    : name(std::move(name)) {

}


unsigned int gg::Scene::addGameObject(gg::GameObject *gameobject) {
    gameobjects.push_back(gameobject);
    return gameobjects.size() - 1;
}

void gg::Scene::removeGameObject(unsigned int index) {
    gameobjects.erase(gameobjects.begin() + index);
}

gg::GameObject *&gg::Scene::getGameObject(unsigned int index) {
    return gameobjects.at(index);
}


std::vector<gg::GameObject*> &gg::Scene::getGameObjects() {
    return gameobjects;
}



// SceneManager
gg::SceneManager::SceneManager() = default;


void gg::SceneManager::addScene(gg::Scene *scene) {
    scenes.insert(std::pair<std::string, Scene*>{scene->name, scene});
}

void gg::SceneManager::removeScene(const std::string& scene_name) {
    scenes.erase(scene_name);
}

gg::Scene *&gg::SceneManager::getScene(const std::string& scene_name) {
    return scenes.at(scene_name);
}


std::map<std::string, gg::Scene*> &gg::SceneManager::getScenes() {
    return scenes;
}
