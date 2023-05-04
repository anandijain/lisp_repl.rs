import readline


def tokenize(s):
    return s.replace('(', ' ( ').replace(')', ' ) ').split()


def parse(tokens):
    if not tokens:
        raise SyntaxError("Unexpected EOF")
    token = tokens.pop(0)
    if token == '(':
        expr = []
        while tokens[0] != ')':
            expr.append(parse(tokens))
        tokens.pop(0)
        return expr
    elif token == ')':
        raise SyntaxError("Unexpected )")
    elif token == "'":
        expr = parse(tokens)
        return ['quote', expr]
    else:
        try:
            return int(token)
        except ValueError:
            try:
                return float(token)
            except ValueError:
                return str(token)


def eval_ast(ast, env):
    if type(ast) == str:
        return env[ast]
    elif type(ast) in (int, float):
        return ast
    elif ast[0] == 'define':
        if type(ast[1]) == list:
            # function definition
            func_name = ast[1][0]
            arg_names = ast[1][1:]
            body = ast[2]

            def func(*args):
                local_env = env.copy()
                local_env.update(zip(arg_names, args))
                return eval_ast(body, local_env)
            env[func_name] = func
        else:
            # variable definition
            key, val = ast[1], ast[2]
            env[key] = eval_ast(val, env)
        return None
    elif ast[0] == 'lambda':
        arg_names, body = ast[1], ast[2]
        print(arg_names, body)

        def func(*args):
            local_env = env.copy()
            local_env.update(zip(arg_names, args))
            return eval_ast(body, local_env)
        return func
    elif ast[0] == 'if':
        cond, true_expr, false_expr = ast[1], ast[2], ast[3]
        return eval_ast(true_expr, env) if eval_ast(cond, env) else eval_ast(false_expr, env)
    elif ast[0] == 'quote':
        return ast[1]
    else:
        fun = eval_ast(ast[0], env)
        args = [eval_ast(arg, env) for arg in ast[1:]]
        return fun(*args)


def repl(env):
    current_input = ""
    parenthesis_count = 0

    def input_is_complete(input_line):
        nonlocal parenthesis_count
        parenthesis_count += input_line.count("(") - input_line.count(")")
        return parenthesis_count == 0

    while True:
        try:
            line = input("LISP> " if not current_input else ".... ")
            if line.strip() == "" and current_input.strip() == "":
                continue

            current_input += line

            # Check if the input is complete
            if input_is_complete(line):
                tokens = tokenize(current_input)
                ast = parse(tokens)
                result = eval_ast(ast, env)
                if result is None:
                    if ast[0] == 'define' and type(ast[1]) == list:
                        print(f"Function {ast[1][0]} defined")
                    elif ast[0] == 'define':
                        print(f"Variable {ast[1]} defined")
                else:
                    print(result)

                # Reset for the next input
                current_input = ""

        except (EOFError, KeyboardInterrupt):
            print("\nExiting...")
            break
        except Exception as e:
            print(f"Error: {e}")
            current_input = ""
            parenthesis_count = 0


env = {
    "=": lambda x, y: x == y,
    "+": lambda x, y: x + y,
    "-": lambda x, y: x - y,
    "*": lambda x, y: x * y,
    "/": lambda x, y: x / y,
    "list": lambda *args: list(args),
    ">": lambda x, y: x > y,
    "<": lambda x, y: x < y,
    "car": lambda lst: lst[0],
    "cdr": lambda lst: lst[1:],
    "cons": lambda a, lst: [a] + lst,
    "null?": lambda lst: lst == [],
}


# repl(env)

# -- tests 

# List of test expressions
# test_expressions = [
#     "(+ 1 2)",
#     "(- 3 4)",
#     "(* 5 6)",
#     "(/ 7 8)",
#     "(define x 10)",
#     "(define (square x) (* x x))",
#     "(if (< x 5) (* x 2) (+ x 3))",
#     "(begin (set! x 20) (display x))",
#     # "(let* ((x 1) (y 2)) (+ x y))",
#     "(vector 1 2 3)",
#     "(vector-ref v 0)",
#     "(for-each display '(1 2 3))",
#     "(lambda (x y) (+ x y))",
# ]

# Test parsing of the expressions
# for expression in test_expressions:
#     test_expression(parser, expression)


# (define x 10) # 10

# (define (square x) (* x x))

# (square 10)

# (define (f x) (* -1 x))

# (define (step f x dt) (+ x (* dt (f x))))

# (step f 3 0.1)

# (step f (step f (step f 3 0.1) 0.1) 0.1)

# (define (factorial n)
#   (if (= n 1)
#       1
#       (* n (factorial (- n 1)))
#   )
# )

# (factorial 5)

# (define (sum_of_squares a b)
#   (+ (square a) (square b))
# )

# (sum_of_squares 3 4)

# (define (sum-of-squares lst)
#   (if (null? lst)
#       0
#       (+ (square (car lst)) (sum-of-squares (cdr lst)))))

# (sum-of-squares '(3 4))


# (define (solve f x0 t0 dt n)
#   ((lambda (euler-helper)
#       (euler-helper euler-helper x0 t0 0))
#    (lambda (euler-helper x t steps)
#       (if (= steps n)
#           x
#           (euler-helper euler-helper (+ x (* dt (f x))) (+ t dt) (+ steps 1))))))

# (solve f 10 0 0.1 50)


# (define (apply-fn f a) (f a))
# (apply-fn (lambda (x) (* x x)) 5) # Should return 25



# (car '(1 2 3))         # ; Should return 1
# (cdr '(1 2 3))         # ; Should return (2 3)
# (cons 0 '(1 2 3))      # ; Should return (0 1 2 3)
# (null? '())            # ; Should return True
# (null? '(1 2 3))       # ; Should return False



# (define (solve f x0 t0 dt n)
#   ((lambda (euler-helper)
#       (euler-helper euler-helper x0 t0 0 '()))
#    (lambda (euler-helper x t steps acc)
#       (if (>= steps n)
#           (reverse acc)
#           (euler-helper euler-helper 
#                         (+ x (* dt (f x))) 
#                         (+ t dt) 
#                         (+ steps 1) 
#                         (cons x acc))))))

# (define (solve f x0 t0 dt n)
#   ((lambda (euler-helper)
#       (euler-helper euler-helper x0 t0 0 '()))
#    (lambda (euler-helper x t steps acc)
#       (if (= steps n)
#           acc
#           (euler-helper euler-helper 
#                         (+ x (* dt (f x))) 
#                         (+ t dt) 
#                         (+ steps 1) 
#                         (cons x acc))))))

# (solve f 10 0 0.1 50)

# # dont work


# (define (sum_of_squares a b)
#   (+ (square a) (square b))
# )

# (sum_of_squares 3 4) # None 



 
# (define (solve f x0 t0 dt n)
#   (define (euler-helper x t steps)
#     (if (= steps n)
#         (list x)
#         (euler-helper (+ x (* dt (f x))) (+ t dt) (+ steps 1)))
#   )
#   (euler-helper x0 t0 0)
# )

# (solve f 10 0 0.1 50)

# (define (step_loop f x dt n)
#   (define (iteration acc current step_count)
#     (if (= step_count n)
#         acc
#         (let ((next_step (step f current dt)))
#           (iteration (cons next_step acc) next_step (+ step_count 1))))
#   )
#   (iteration (list x) x 0)
# )
