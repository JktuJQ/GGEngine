#include "../headers/components.h"

#include <utility>



// Component
gg::Component::Component(std::string name, std::string tag)
    : name(std::move(name)), tag(std::move(tag)) {

}

gg::Component::~Component() = default;


// BoxCollider
gg::BoxCollider::BoxCollider(std::string name, gg::Rect2D rect, std::string tag)
    : Component(std::move(name), std::move(tag)), rect(std::move(rect)) {

}


void gg::BoxCollider::move_on(gg::Vector2D vector) {
    rect.move_on(vector);
}

void gg::BoxCollider::move_to(gg::Point2D point) {
    rect.move_to(point);
}

gg::Rect2D &gg::BoxCollider::getRect() {
    return rect;
}



// Animation
gg::Sprite::Animation::Animation(gg::Sprite *sprite, std::map<std::string, std::vector<Image>> images)
    : binded_sprite(sprite), packed_images(std::move(images)) {
    animation_state = AnimationState::DISABLED;
}


void gg::Sprite::Animation::startAnimation(int delay) {
    if (animation_state == AnimationState::ENABLED){
        throw GGEngineError("Failed to start animation, cause it was already enabled");
    }
    animation_state = AnimationState::ENABLED;
    timer.setInterval(&Animation::animate, delay);
}

void gg::Sprite::Animation::animate() {
    if (animation_state == AnimationState::ENABLED){
        binded_sprite->setImage(packed_images.at(status).at(0));
        std::rotate(packed_images.at(status).begin(), packed_images.at(status).begin() + 1, packed_images.at(status).end());
    }
}

void gg::Sprite::Animation::stopAnimation() {
    if (animation_state == AnimationState::DISABLED){
        throw GGEngineError("Failed to stop animation, cause it wasn't enabled");
    }
    timer.stop();
}

gg::Sprite::Animation::AnimationState gg::Sprite::Animation::getAnimationState() {
    return animation_state;
}



// Sprite
gg::Sprite::Sprite(std::string name, gg::ITexture *texture, std::string tag)
    : Component(std::move(name), std::move(tag)), texture(texture) {

}


void gg::Sprite::move_on(gg::Vector2D vector) {
    texture->move_on(vector);
}

void gg::Sprite::move_to(gg::Point2D point) {
    texture->move_to(point);
}

void gg::Sprite::setImage(gg::Image image) {
    texture->setImage(image);
}

void gg::Sprite::render() {
    texture->render();
}


void gg::Sprite::setAnimation(gg::Sprite::Animation new_animation) {
    animation = std::move(new_animation);
}

gg::Sprite::Animation &gg::Sprite::getAnimation() {
    return animation;
}
