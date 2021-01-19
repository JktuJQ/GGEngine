#ifndef GGENGINE_CORE_H
#define GGENGINE_CORE_H

// Includes
#include "declarations.h"
#include "data.h"



namespace gg {

    class IMovable {
        // IMovable interface for moving objects
    public:
        virtual void move_on(Vector2D vector) = 0; // Moves object on vector
        virtual void move_to(Point2D point) = 0; // Moves object to position
    };



    class ITexture : public IMovable {
        // Base texture class for displaying images. Must be inherited
    public:
        virtual void setImage(Image image) = 0;
        virtual void render() = 0;
    };



    class Timer final {
        // Timer with JavaScript interface
    public:
        template<typename Function>
        void setTimeout(Function function, int delay); // Invokes function after delay (in ms)
        template<typename Function>
        void setInterval(Function function, int interval); // Invokes function every time after delay (in ms)
        void stop();  // Stops timer

    private:
        bool clear = false;
    };


    class Vector2D {
    public:
        // Base 2D vector that is used to move different objects from their positions
        int offset_x, offset_y;


        Vector2D(int offset_x, int offset_y);

        [[nodiscard]] Vector2D operator - () const; // Makes a vector negative by making both offsets negative
        [[nodiscard]] Vector2D operator ~ () const; // Makes a vector inverted by swapping offsets
        Vector2D operator + (Vector2D other_vector) const; // Adds vectors by adding their offsets
        Vector2D operator - (Vector2D other_vector) const; // Subtracts vectors by subtracting their offsets
        void operator += (Vector2D other_vector);
        void operator -= (Vector2D other_vector);
    };



    class Point2D : public IMovable {
        // Base 2D point on surface with integer x and y, implements IMovable
    public:
        int x, y;


        Point2D(int x, int y);

        void move_on(Vector2D vector) override;
        void move_to(Point2D point) override;
    };



    class Rect2D : public IMovable {
        // Rectangle on surface, used to check for collisions, implements IMovable
    public:
        Point2D point_ul, point_br;


        Rect2D(Point2D point_ul, int width, int height);
        Rect2D(Point2D point_ul, Point2D point_br);

        void move_on(Vector2D vector) override;
        void move_to(Point2D point) override;

        [[nodiscard]] int width() const; // Returns width of rectangle
        [[nodiscard]] int height() const; // Returns height of rectangle

        [[nodiscard]] bool intersects_with(const Rect2D& other_rect) const; // Checks intersection between two rects

        static Vector2D difference(const Point2D& point1, const Point2D& point2); // Returns width and height of rect, where point1 is ul point, point2 is br point
    };

}



#endif
