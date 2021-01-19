#include "../headers/core.h"



// Timer
template<typename Function>
void gg::Timer::setTimeout(Function function, int delay) {
    this->clear = false;
    std::thread t([=, this]() {
        if(this->clear) return;
        std::this_thread::sleep_for(std::chrono::milliseconds(delay));
        if(this->clear) return;
        function();
    });
    t.detach();
}

template <typename Function>
void gg::Timer::setInterval(Function function, int interval) {
    this->clear = false;
    std::thread t([=, this]() {
        while(true) {
            if(this->clear) return;
            std::this_thread::sleep_for(std::chrono::milliseconds(interval));
            if(this->clear) return;
            function();
        }
    });
    t.detach();
}

void gg::Timer::stop() {
    clear = true;
}


// Vector2D
gg::Vector2D::Vector2D(int offset_x, int offset_y)
    : offset_x(offset_x), offset_y(offset_y) {

}


gg::Vector2D gg::Vector2D::operator - () const {
    return Vector2D(-offset_x, -offset_y);
}

gg::Vector2D gg::Vector2D::operator ~ () const {
    return gg::Vector2D(offset_y, offset_x);
}

gg::Vector2D gg::Vector2D::operator + (gg::Vector2D other_vector) const {
    return gg::Vector2D(offset_x + other_vector.offset_x, offset_y + other_vector.offset_y);
}

gg::Vector2D gg::Vector2D::operator - (gg::Vector2D other_vector) const {
    return gg::Vector2D(offset_x - other_vector.offset_x, offset_y - other_vector.offset_y);
}

void gg::Vector2D::operator += (gg::Vector2D other_vector) {
    offset_x += other_vector.offset_x;
    offset_y += other_vector.offset_y;
}

void gg::Vector2D::operator -= (gg::Vector2D other_vector) {
    offset_x -= other_vector.offset_x;
    offset_y -= other_vector.offset_y;
}



// Point2D
gg::Point2D::Point2D(int x, int y)
    : x(x), y(y) {

}


void gg::Point2D::move_on(gg::Vector2D vector) {
    x += vector.offset_x;
    y += vector.offset_y;
}

void gg::Point2D::move_to(gg::Point2D point) {
    x = point.x;
    y = point.y;
}



// Rect2D
gg::Rect2D::Rect2D(gg::Point2D point_ul, int width, int height)
    : point_ul(std::move(point_ul)), point_br(point_ul.x + width, point_ul.y + height) {

}

gg::Rect2D::Rect2D(gg::Point2D point_ul, gg::Point2D point_br)
    : point_ul(std::move(point_ul)), point_br(std::move(point_br)) {

}


void gg::Rect2D::move_on(gg::Vector2D vector) {
    point_ul.move_on(vector);
    point_br.move_on(vector);
}

void gg::Rect2D::move_to(gg::Point2D point) {
    point_ul.move_to(point);
    Vector2D br_offset = Vector2D(width(), height());
    point_br.move_to(point);
    point_br.move_on(br_offset);
}


int gg::Rect2D::width() const {
    return point_br.x - point_ul.x;
}

int gg::Rect2D::height() const {
    return point_br.y - point_ul.y;
}


bool gg::Rect2D::intersects_with(const gg::Rect2D& other_rect) const {
    return (point_ul.x <= other_rect.point_ul.x <= point_br.x or other_rect.point_ul.x <= point_ul.x <= other_rect.point_br.x)
        and (point_ul.y <= other_rect.point_ul.y <= point_br.y or other_rect.point_ul.y <= point_ul.y <= other_rect.point_br.y);
}


gg::Vector2D gg::Rect2D::difference(const gg::Point2D &point1, const gg::Point2D &point2) {
    Rect2D rect = Rect2D(point1, point2);
    return Vector2D(rect.width(), rect.height());
}
