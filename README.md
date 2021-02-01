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

It uses an intermediate language for easy conversion to different languages, such as C++, Python. 
A standard parser that decomposes the code into a semantic tree is attached.

Hints of building your own **GGEngine Designer** .exe file are written in "*designer/README.md*".

## GGEngine Parser
**GGEngine Parser** files are located in *parser* directory.
It implements simple intermediate language .gg, 
which can be converted in GGEngine library syntax on every programming language.

Base parser can convert in C++, but it's easy to apply converting rules in parser.
Almost every language that has GGEngine library will be supported to conversion.

You can check examples of parser work and .gg syntax in source files.
Hints how to use **parser** or how to apply new converting rules are written in "*parser/README.md*".

----------------------------------------------
This C++20 library was created at **10.01.21**, by **JktuJQ**.