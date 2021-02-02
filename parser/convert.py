"""
Python script for converting parsed .gg files in other programming languages by using rules
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
"""

from typing import List, Dict, Union
import sys


def convert(argc: int, argv: List[str]):
    if argc == 1:
        print("No given language to convert and no file to convert chosen")
        sys.exit(-1)
    elif argc == 2:
        print("No file to convert chosen")
        sys.exit(-1)

    from language_rules import rules
    if argv[1] not in rules.keys():
        print("Can't convert in that language")
        sys.exit(-1)
    rule: Dict[str, Union[str, Dict[str, str]]] = rules[argv[1]]

    filename = argv[2]

    parsed_file = open(filename, mode="r")
    output_text = ""

    output_text += rule["start"]
    for line in parsed_file.readlines():
        code_type = line.split()[0]
        if code_type == "var":
            converted_string = rule["code"]["var"]
            var_type = line.split()[1]
            var_name = line.split()[2]
            class_name = line.split()[3]
            args = list(eval(" ".join(line.split()[4:])))
            converted_string = converted_string.format(var_type=var_type,
                                                       var_name=var_name,
                                                       class_name=class_name,
                                                       args=", ".join(args))
            output_text += converted_string

        elif code_type == "method":
            converted_string = rule["code"]["method"]
            var_name = line.split()[1]
            method_name = line.split()[2]
            args = list(eval(line.split()[3]))
            converted_string = converted_string.format(var_name=var_name,
                                                       method_name=method_name,
                                                       args=", ".join(args))
            output_text += converted_string
    output_text += rule["end"]
    parsed_file.close()

    converted_file = open(filename, mode="w")
    converted_file.write(output_text)
    converted_file.close()

    sys.exit(0)


if __name__ == "__main__":  # When executed as script, not imported
    convert(len(sys.argv), sys.argv)
