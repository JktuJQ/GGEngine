GGEngine
========

**GGEngine** - C++20 game engine compatible with any GUI library turned into 
wrapper classes.

> The main purpose of this game engine - make developing 2D games on C++ easier.
> While GGEngine takes care of all the logic of the game, the user is left 
to draw on the screen.

### Architecture
Architecture of **GGEngine** is basically the same as in Unity game engine but with some concepts.
Game levels are represented by *Scene*, while the levels themselves are comprised of *GameObject*. 
The game object consists of *Components*, which in turn implement the game logic. 
Also, the game logic is represented by *Event*. 
They call slot functions when *Event* invoked. 
All *Components* processing is done by *Processor*. 
To implement custom logic, you need to create a custom *Component* and, 
in a class that inherits *Processor*, define the processing of this *Component*.

## Designer
**GGEngine Designer** files are located in *designer* directory. 
It helps creating *Scenes* with placing all *GameObjects*, *Sprites* and *Colliders*.

It uses an intermediate language for easy conversion to different languages, such as Python. 
A standard parser that decomposes the code into a semantic tree is attached.

----------------------------------------------
This C++20 library was created at **10.01.21**, by **JktuJQ**.