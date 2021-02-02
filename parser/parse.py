"""
Python script for parsing .gg files
Second system arg must be a name of file to parse, third system arg must be a name of output file
Fourth system argument can be the name of the programming language to be converted to (need to be in language_rules.py.rules.keys());
    default='txt' which means nothing will be converted;
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
"""

from typing import List
import re
import sys

__comments_pattern = re.compile(r"(\".*?(?<!\\)\"|\'.*?(?<!\\)\')|(/\*.*?\*/|//[^\r\n]*$)", re.MULTILINE | re.DOTALL)
__args_pattern = re.compile(r"^\(\)$", re.DOTALL)


def parse(argc: int, argv: List[str]):
    convert_to = "txt"

    if argc == 1:
        print("No convertible file specified and no file specified for output")
        sys.exit(-1)
    elif argc == 2:
        print("No file specified for output")
        sys.exit(-1)
    elif argc == 4:
        from language_rules import rules
        if argv[3] in rules.keys() or argv[3] == "txt":
            convert_to = argv[3]
        else:
            print(f"Can't convert in {argv[3]} language")
            sys.exit(-1)

    try:
        parse_file = open(argv[1], mode="r")
        output_file = open(argv[2], mode="w")

        output = list()

        full_text = parse_file.read()
        text_without_comments = __comments_pattern.sub(
            lambda match: "" if match.group(2) is not None else match.group(1), full_text)
        code_lines = list(filter(lambda x: x, text_without_comments.split("\n")))
        for line in code_lines:
            try:
                if " ::= " in line:  # Declaring a variable
                    var_type = line.split(" - ")[0]
                    var_name = line.split(" - ")[1].split(" ::= ")[0]
                    class_name = line.split(" ::= ")[1].split(" (")[0]
                    args_string = " ".join(line.split(" ::= ")[1].split()[1:])
                    args = [] if not args_string else re.sub(__args_pattern, "", args_string[1:-1]).split(" ")
                    output.append(f"var {var_type} {var_name} {class_name} {args}\n")
                elif " -> " in line:  # Method called
                    var_name = line.split(" -> ")[0]
                    method_name = line.split(" -> ")[1].split(" (")[0]
                    args_string = " ".join(line.split(" -> ")[1].split()[1:])
                    args = [] if not args_string else re.sub(__args_pattern, "", args_string[1:-1]).split(" ")
                    output.append(f"method {var_name} {method_name} {args}\n")
            except IndexError:
                line_index = full_text.split("\n").index(line) + 1
                print(f"SyntaxError at line {line_index}")
                sys.exit(-1)

        # Convert
        output_file.writelines(output)

        parse_file.close()
        output_file.close()

        if convert_to != "txt":
            from convert import convert
            convert(3, ["convert.py", convert_to, argv[2]])

    except FileNotFoundError:
        print("File is not found")
        sys.exit(-1)

    finally:
        try:
            parse_file.close()
            output_file.close()

        except UnboundLocalError:
            pass

        sys.exit(0)


if __name__ == "__main__":  # When executed as script, not imported
    parse(len(sys.argv), sys.argv)
