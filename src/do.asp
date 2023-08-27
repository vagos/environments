dim(1..10).

1 { place(O, X, Y, Z) : dim(X), dim(Y), dim(Z) } 1 :- object(O).

1 { floor(X, Y, 0) } 1 :- dim(X), dim(Y).

% :- 2 { place(_, X, Y, Z); place(_, X, Y, Z) }, floor(X, Y, Z).

in(O, IX, IY, IZ) :- place(O, X, Y, Z), size(O, SX, SY, SZ), 
                     X <= IX, X + SX > IX,
                     Y <= IY, Y + SY > IY,
                     Z <= IZ, Z + SZ > IZ,
                     dim(IX), dim(IY), dim(IZ).

% O1 is on O2
on(O1, O2) :- in(O2, X, Y, Z), size(O2, SX, SY, SZ), in(O1, X, Y, Z + SZ).

:- in(O1, X, Y, Z), in(O2, X, Y, Z), O1 != O2.
:- place(O, X, Y, Z), Z > 1, 0 { in(OO, X, Y, Z - 1) } 0.
:- type(T, tree), object(O), on(O, T).

#show place/4.
% #show on/2.
