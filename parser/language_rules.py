"""
This file describes all rules of converting .gg file in other languages
Rules are described by using this variables by writing them in '{}'
Variables:
    'var_type' - the name of type represented as string (only if declaring variable [var])
    'var_name' - the name of variable represented as string
    'class_name' - the name of class represented as string (only if declaring variable [var])
    'method_name' - the name of method represented as string (only if calling a method [method])
    'args' - list of all arguments represented as strings (it will be always replaced as ', '.join(args))
"""

rules = {"cpp": {"start": "#include <GGEngine/headers/all.h>\nusing namespace gg;\n\nclass Game {\npublic:",
                 "code": {"var": "\n\t{var_type}* {var_name} = new {class_name}({args});",
                          "method": "\n\t{var_name}->{method_name}({args});"},
                 "end": "\n};"}}
