fun fib n = case n of
                0 => 1
              | 1 => 1
              | n =>  fib (n - 1) + fib (n - 2)
val a = print (fib 0)
val a = print (fib 1)
val a = print (fib 2)
val a = print (fib 3)
val a = print (fib 4)
val a = print (fib 5)
