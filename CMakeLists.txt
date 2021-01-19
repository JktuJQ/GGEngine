cmake_minimum_required(VERSION 3.16)
project(GGEngine)
set(CMAKE_CXX_STANDARD 20)

add_library(${PROJECT_NAME} STATIC
        headers/all.h
        headers/declarations.h

        headers/errors.h
        headers/data.h
        headers/core.h
        headers/events.h
        headers/components.h
        headers/gameobjects.h
        headers/scenes.h
        headers/processor.h

        sources/errors.cpp
        sources/data.cpp
        sources/core.cpp
        sources/events.cpp
        sources/components.cpp
        sources/gameobjects.cpp
        sources/scenes.cpp
        sources/processor.cpp)