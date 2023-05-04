## dew 

(define x 5)
x # segfaults now 

note multiline repl pasting doesn't work
// doesn't work yet 
<!-- (step f 3.0 0.1) -->

(define (f x) (* -1 x))
(define (step f x dt) (+ x (* dt (f x))))
```julia
julia> f(x) = -x
f (generic function with 1 method)

julia> step(f, x, dt) = x + dt*f(x)
step (generic function with 1 method)


julia> @code_llvm optimize=false step(f, 3.0, 0.1)
;  @ REPL[14]:1 within `step`
```
```llvm
define double @julia_step_3438(double %0, double %1) #0 {
top:
  %2 = call {}*** @julia.get_pgcstack()
  %3 = bitcast {}*** %2 to {}**
  %current_task = getelementptr inbounds {}*, {}** %3, i64 -13
  %4 = bitcast {}** %current_task to i64*
  %world_age = getelementptr inbounds i64, i64* %4, i64 14
; ┌ @ REPL[13]:1 within `f`
; │┌ @ float.jl:406 within `-`
    %5 = fneg double %0
; └└
; ┌ @ float.jl:410 within `*`
   %6 = fmul double %1, %5
; └
; ┌ @ float.jl:408 within `+`
   %7 = fadd double %0, %6
; └
  ret double %7
}
```
```

mylisp[HIST:100 | LOOP: 11]>> (step f 3.0 0.1)

List([Symbol("step"), Symbol("f"), Float(3.0), Float(0.1)])
Instruction does not dominate all uses!
  %f1 = alloca double, align 8
  %f = load double, double* %f1, align 8
Instruction does not dominate all uses!
  %f1 = alloca double, align 8
  %f1 = load double, double* %f1, align 8
Error: Invalid generated function.

// this works 
(define (step x dt) (+ x (* dt (* -1 x)))) 
(step (step (step 3 0.1) 0.1) 0.1) // 2.187...

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

```scheme
(define (euler f x0 t0 dt n)
  (define (euler-helper x t steps)
    (if (= steps n)
        (list x)
        (begin
          (set! x (f x t))
          (set! t (+ t dt))
          (euler-helper x t (+ steps 1))
        )
    )
  )
  (euler-helper x0 t0 0)
)

(define (lorenz x t)
  (let* ((sigma 10.0)
         (rho 28.0)
         (beta 8/3)
         (x0 (vector-ref x 0))
         (y0 (vector-ref x 1))
         (z0 (vector-ref x 2))
         (dxdt (* sigma (- y0 x0)))
         (dydt (- (* x0 (- rho z0)) y0))
         (dzdt (- (* x0 y0) (* beta z0))))
    (vector (+ x0 (* dt dxdt))
            (+ y0 (* dt dydt))
            (+ z0 (* dt dzdt)))
  )
)

(define dt 0.01)
(define n 10000)
(define x0 (vector 1.0 1.0 1.0))

(define result (euler lorenz x0 0 dt n))

(for-each
  (lambda (x)
    (display (vector-ref x 0))
    (display " ")
    (display (vector-ref x 1))
    (display " ")
    (display (vector-ref x 2))
    (newline))
  result)
```