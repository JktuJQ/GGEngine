GGParser
========

**GGParser** -  a couple of python scripts which are implementing .gg intermediate language
and a parser of it.
> The main purpose of GGParser - make an easy way to share code between different programming
> languages. The user can write one .gg file code and then convert it to as many programming languages
> as he want.

## .gg file
**.gg file** has intuitive syntax.
"*//*" is an one-line comment, "*/\* ... \*/*" is a multi-line comment.
###### Declaring a variable:
First you need to write type of variable, for example Scene (*"Scene"*)

Then, after *" - "* write name of variable, for example scene (*"Scene - scene"*)

Parser detects declaring a variable only if *" ::= "* string is in the line. So it's an assignment
operator in syntax (*"Scene - scene ::= "*)

Next step is writing name of class which instance must be created, usually it's the same as 
type of variable (*"Scene - scene ::= Scene"*)

On last step you need to write all arguments you want to pass in variable declaration.
They must be separated with *", "* and must be in parentheses. You can skip that step if 
nothing is passed in arguments (*"Scene - scene ::= Scene ("scene_name)"*)

###### Calling a method
First you need to write name of variable which method you want to call, for example scene (*"scene"*)

Parser detects calling a method only if *"->"* string is in the line. So it's a member access operator
in syntax (*"scene->"*)

Next step is writing name of method you want to call, for example addGameObject (*"scene->addGameObject"*)

Last step is the same as in declaring variable - arguments. You still can skip that step if nothing
is passed to function (*"scene->addGameObject (hero)"*)

## parse.py
**parse.py** is a python script that decomposes .gg intemediate language into semantic tree.
It takes at least 2 system argument besides the filename:
* A name of file to parse ("*example.gg*")
* A name of output file ("*example.txt*")

The third argument can be the name of the language to convert to immediately.
This name need to be in *language_rules.py.rules.keys()*, otherwise error will be called.
Default value of third argument is "txt", which means that nothing will be converted.

Example:

    python parse.py example.gg example.txt
    
    (or python parse.py example.gg example.txt cpp)
    
example.txt:

    var SceneManager* sceneManager SceneManager []
    var Scene* scene Scene ['"scene1"']
    var GameObject* hero GameObject ['"hero_name"', '"new_tag"']
    method scene addGameObject ['hero']
    method sceneManager addScene ['scene']
    
    (or
    #include <GGEngine/headers/all.h>
    using namespace gg;
    
    class Game {
    public:
        SceneManager* sceneManager = new SceneManager();
        Scene* scene = new Scene("scene1");
        GameObject* hero = new GameObject("hero_name", "new_tag");
        scene->addGameObject(hero);
        sceneManager->addScene(scene);
    };)


## convert.py
**convert.py** is a python script that converts parsed .gg file into programming language by using rules
that are given in *language_rules.py.rules*.
It takes two system arguments besides the filename:
* A name of file to be converted (converted text will be saved in the same file)
* A name of programming language to convert in (name string must be in *language_rules.py.rules.keys()*)

Example:

    python convert.py cpp example.txt

example.txt:

    #include <GGEngine/headers/all.h>
    using namespace gg;
    
    class Game {
    public:
        SceneManager* sceneManager = new SceneManager();
        Scene* scene = new Scene("scene1");
        GameObject* hero = new GameObject("hero_name", "new_tag");
        scene->addGameObject(hero);
        sceneManager->addScene(scene);
    };
    
## language_rules.py
**language_rules.py** is a python file with *rule* dictionary
that describes all converting operations.
*rule* dictionary consists of:
* string key - name of rule (name of programming language)
* dict value - dictionary of rules:
  * string key - name of converting part ('start', 'code' or 'end')
  
  code part is a dict:
    * string key - is a var declared or a method called in the code string ('var' or 'method')
    * string value - string that describes syntax of conversion, must be a string that can be formatted

--------
Those python scripts were created at **02.02.2021**, by **JktuJQ**.
