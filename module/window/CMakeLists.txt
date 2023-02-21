cmake_minimum_required(VERSION 3.10)

project(Window CXX)

# This is so SDL2 shared objects are placed together with our binary
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR})
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR})

add_subdirectory(SDL2)

add_executable(Window window.cpp)
target_include_directories(Window PUBLIC SDL2/include)
target_link_libraries(Window PUBLIC SDL2 SDL2main)
