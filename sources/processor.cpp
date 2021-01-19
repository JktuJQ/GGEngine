#include "../headers/processor.h"



// Processor
void gg::Processor::process(gg::Scene *scene) {

    std::map<GameObject*, BoxCollider*> box_colliders;
    std::vector<Sprite*> sprites;

    for (GameObject *gameobject : scene->getGameObjects()){
        for (const auto& component_pair : gameobject->getComponents()){

            // BoxColliders
            if (auto *box_collider = dynamic_cast<BoxCollider*>(component_pair.second)){
                box_colliders.insert(std::pair<GameObject*, BoxCollider*>{gameobject, box_collider});
            }

            // Sprites
            if (auto *sprite = dynamic_cast<Sprite*>(component_pair.second)){
                sprites.push_back(sprite);
            }

        }
    }

    // BoxColliders
    for (std::pair<GameObject*, BoxCollider*> box_collider_pair1 : box_colliders){
        for (std::pair<GameObject*, BoxCollider*> box_collider_pair2 : box_colliders){
            if (box_collider_pair1.first == box_collider_pair2.first) continue;
            box_collider_pair1.first->getEventHandler().invokeEvent("on_collision", box_collider_pair2.first);
            box_collider_pair2.first->getEventHandler().invokeEvent("on_collision", box_collider_pair1.first);
        }
    }

    // Sprites
    for (Sprite *sprite : sprites){
        sprite->render();
    }

}
