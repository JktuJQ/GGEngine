GGParser
========

## parse.py
**parse.py** is a python script that decomposes .gg intemediate language into semantic tree.
It takes at least 2 system argument besides the file name:
* A name of file to parse ("*example.gg*")
* A name of output file ("*example.txt*")

The third argument can be the name of the language to convert to immediately.
This name need to be in language_rule.py.rules.keys(), otherwise error will be called.
Default value of third argument is "txt", which means that nothing will be converted.

Example:

    python parse.py example.gg example.txt
 
example.txt:

    var SceneManager* sceneManager SceneManager []
    var Scene* scene Scene ['"scene1"']
    var GameObject* hero GameObject ['"hero_name"', '"new_tag"']
    method scene addGameObject ['hero']
    method sceneManager addScene ['scene']

--------
Those python scripts were created at ** **, by **JktuJQ**.
