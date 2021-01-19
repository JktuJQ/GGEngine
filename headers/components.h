#ifndef GGENGINE_COMPONENTS_H
#define GGENGINE_COMPONENTS_H

// Includes
#include "declarations.h"
#include "errors.h"
#include "core.h"



namespace gg {

    class Component {
        // Base component class, inherited classes implement logic of gameobject
    public:
        const std::string name, tag;


        Component(std::string name, std::string tag = DEFAULT_TAG);
        virtual ~Component();

    protected:
        inline static std::string DEFAULT_TAG = "component";
    };



    class BoxCollider final : public Component, public IMovable {
        // BoxCollider component for collision checks
    public:
        BoxCollider(std::string name, Rect2D rect, std::string tag = DEFAULT_TAG);

        void move_on(Vector2D vector) override;
        void move_to(Point2D point) override;

        Rect2D &getRect();

    private:
        Rect2D rect;

    protected:
        inline static std::string DEFAULT_TAG = "box_collider";
    };



    class Sprite final : public Component, public ITexture {
        // Sprite component for visualizing images on surface
    public:
        class Animation final {
            // Animation for Sprite component
        public:
            Sprite *binded_sprite;
            std::string status;

            enum class AnimationState { DISABLED = 0, ENABLED = 1 };


            Animation(Sprite *sprite, std::map<std::string, std::vector<Image>> images);

            void startAnimation(int delay = 100); // Starts animation, delay is in milliseconds
            void animate(); // Animates sprite
            void stopAnimation(); // Stops animation

            AnimationState getAnimationState();

        private:
            std::map<std::string, std::vector<Image>> packed_images;
            Timer timer;
            AnimationState animation_state;
        };


        ITexture *texture;


        Sprite(std::string name, ITexture *texture, std::string tag);

        void move_on(Vector2D vector) override;
        void move_to(Point2D point) override;
        void setImage(Image image) override;
        void render() override;

        void setAnimation(Animation new_animation); // Sets new animation to sprite

        Animation &getAnimation();

    private:
        Animation animation {this, {}};

    protected:
        inline static std::string DEFAULT_TAG = "sprite";
    };

}



#endif
