cmake_minimum_required(VERSION 3.16)
project(designer)
set(CMAKE_CXX_STANDARD 20)


add_executable(${PROJECT_NAME} WIN32 main.cpp

        "uic/BaseUi.h"
        "uic/MainWindowUi.h" "uic/MainWindowUi.cpp"
        "uic/NewProjectDialogUi.h" "uic/NewProjectDialogUi.cpp"

        forms/MainWindow.h forms/MainWindow.cpp
        forms/NewProjectDialog.h forms/NewProjectDialog.cpp)



# Qt library
set(CMAKE_INCLUDE_CURRENT_DIR ON)
set(CMAKE_PREFIX_PATH ../../../Qt/5.9.9/mingw53_32/)  # Change path to your Qt compiler

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTORCC ON)

find_package(Qt5Widgets REQUIRED)
target_link_libraries(${PROJECT_NAME} Qt5::Widgets)



# GGEngine library
find_library(GGLibrary GGEngine data/library)  # Place compiled library file (.a) in data/library folder
include_directories(../headers)
target_link_libraries(${PROJECT_NAME} ${GGLibrary})
