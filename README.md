## dew 


intrinsics to test 

* llvm.fabs: Absolute value of a floating-point number.
* llvm.sqrt: Square root of a floating-point number.
* llvm.sin: Sine of a floating-point number.
* llvm.cos: Cosine of a floating-point number.
* llvm.tan: Tangent of a floating-point number.
* llvm.asin: Arcsine of a floating-point number.
* llvm.acos: Arccosine of a floating-point number.
* llvm.atan: Arctangent of a floating-point number.
* llvm.atan2: Arctangent of two floating-point numbers (y/x).
* llvm.exp: Exponential function (e^x) of a floating-point number.
* llvm.exp2: Base-2 exponential function (2^x) of a floating-point number.
* llvm.log: Natural logarithm (base e) of a floating-point number.
* llvm.log2: Base-2 logarithm of a floating-point number.
* llvm.log10: Base-10 logarithm of a floating-point number.
* llvm.ceil: Ceiling function of a floating-point number.
* llvm.floor: Floor function of a floating-point number.
* llvm.round: Round function of a floating-point number.
* llvm.trunc: Truncate function of a floating-point number.


(llvm.sqrt (llvm.fabs x))

sad, so be it 


Variables
constexpr double 	e = 2.7182818284590452354
 
constexpr double 	egamma = .57721566490153286061
 
constexpr double 	ln2 = .69314718055994530942
 
constexpr double 	ln10 = 2.3025850929940456840
 
constexpr double 	log2e = 1.4426950408889634074
 
constexpr double 	log10e = .43429448190325182765
 
constexpr double 	pi = 3.1415926535897932385
 
constexpr double 	inv_pi = .31830988618379067154
 
constexpr double 	sqrtpi = 1.7724538509055160273
 
constexpr double 	inv_sqrtpi = .56418958354775628695
 
constexpr double 	sqrt2 = 1.4142135623730950488
 
constexpr double 	inv_sqrt2 = .70710678118654752440
 
constexpr double 	sqrt3 = 1.7320508075688772935
 
constexpr double 	inv_sqrt3 = .57735026918962576451
 
constexpr double 	phi = 1.6180339887498948482
 
constexpr float 	ef = 2.71828183F
 
constexpr float 	egammaf = .577215665F
 
constexpr float 	ln2f = .693147181F
 
constexpr float 	ln10f = 2.30258509F
 
constexpr float 	log2ef = 1.44269504F
 
constexpr float 	log10ef = .434294482F
 
constexpr float 	pif = 3.14159265F
 
constexpr float 	inv_pif = .318309886F
 
constexpr float 	sqrtpif = 1.77245385F
 
constexpr float 	inv_sqrtpif = .564189584F
 
constexpr float 	sqrt2f = 1.41421356F
 
constexpr float 	inv_sqrt2f = .707106781F
 
constexpr float 	sqrt3f = 1.73205081F
 
constexpr float 	inv_sqrt3f = .577350269F
 
constexpr float 	phif = 1.61803399F



```
?> def  square(x) x*x
-> Attempting to parse lexed input: 
[Def, Ident("square"), LParen, Ident("x"), RParen, Ident("x"), Op('*'), Ident("x")]

-> Function parsed: 
Function { prototype: Prototype { name: "square", args: ["x"], is_op: false, prec: 0 }, body: Some(Binary { op: '*', left: Variable("x"), right: Variable("x") }), is_anon: false }

-> Expression compiled to IR:define double @square(double %x) {
entry:
  %x1 = alloca double, align 8
  store double %x, double* %x1, align 8
  %x2 = load double, double* %x1, align 8
  %x3 = load double, double* %x1, align 8
  %tmpmul = fmul double %x2, %x3
  ret double %tmpmul
}

?> square(3)
-> Attempting to parse lexed input: 
[Ident("square"), LParen, Number(3.0), RParen]

-> Expression parsed: 
Some(Call { fn_name: "square", args: [Number(3.0)] })

-> Expression compiled to IR:define double @anonymous() {
entry:
  %tmp = call double @square(double 3.000000e+00)
  ret double %tmp
}
=> 9
```

var x, y in 0
-> Expression compiled to IR:define double @anonymous() {
entry:
  %y = alloca double, align 8
  %x = alloca double, align 8
  store double 0.000000e+00, double* %x, align 8
  store double 0.000000e+00, double* %y, align 8
  ret double 0.000000e+00
}
=> 0

(define (add-five x)
  (display "Adding five to ")
  (display x)
  (newline)
  (+ x 5))


(define x 5)
(define (square x) (* x x))
(square x)

```lisp
(define (apply-operation op a b c)
  (op a b c))

(define (sum-cubed a b c)
  (+ (* a a a) (* b b b) (* c c c)))

(define (product a b c)
  (* a b c))

(define higher-order-fn
  (lambda (op a b c)
    (if (= op 1)
        (apply-operation sum-cubed a b c)
        (apply-operation product a b c))))

(define (add1 x) (+ x 1))
(define (doit f x) (f x))
(display (doit add1 1))


(define x 10) ; Global scope

(define (example-function1 y)
  ; Local scope of example-function1
  (define z 5) ; Local variable z inside example-function1
  (+ x y z))   ; Access global x, local y, and local z

(define (example-function2 a)
  ; Local scope of example-function2
  (let ((x 20)  ; Define a new local x which does not affect the global one
        (b 30)) ; Define a local b
    (+ x a b))) ; Access local x, a, and b

(display (example-function1 7)) ; Output: 22 (10 + 7 + 5)
(display (example-function2 5)) ; Output: 55 (20 + 5 + 30)
```