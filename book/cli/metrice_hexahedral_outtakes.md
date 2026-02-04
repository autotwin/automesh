## Deprecated and likely deleted soon

#### Orthocenter

We introduce the concept of orthocenter first for a triangle, and then for a tetrahedron.

An altitude of a triangle is a line from a vertex perpendicular to the opposite edge.  A triangle has three vertices, three edges, and thus three altitude.

An altitude of a tetrahedron is a line from a vertex perpendicular to the opposite face. A tetrahedron has four vertices, four faces, and thus four altitudes. 

* For a **general tetrahedron**, the four altitude typically do **not** all meet at a single point.  In this case, there is no orthocenter.
* For an **orthocentric tetrahedron**, the four altitudes meet at a single, unique point called the orthocenter.   The orthocentric tetrahedron has one and only one orthocenter.

The orthocenter is typically denoted as point $H$ located by vector $h$.

"The **orthocenter** of a triangle $H$ is the point where the three (possibly extended) altitudes intersect.  The orthocenter lies inside the triangle if and only if the triangle is acute.  For a right triangle, the orthocenter coincides with the vertex at the right angle.  For an equilateral triangle, the orthocenter coincides with the centroid."

![triangle_orthocenter](img/triangle_orthocenter.png)

"An **orthocentric tetrahedron** is a tetrahedron where all pairs of opposite edges are perpendicular.  In an orthocentric tetrahedron the four altitudes are concurrent.  This common point is called the **tetrahedron orthocenter**."

The orthocenter of a tetrahedron is a point where all four altitudes meet.  

![tetrahedron_orthocenter](img/tetrahedron_orthocenter.gif)

#### Notes

We define the **orthocenter** $\mathbf{h}$ as the point on the face of the triangle with vertices $\mathbf{a}$, $\mathbf{b}$, and $\mathbf{c}$ where the three altitudes meet.  An altitude is perpendicular to the opposite side.  So the altitude from vertex $\mathbf{a}$ is perpendicular to the opposite side $\mathbf{b} \mathbf{c}$,

$$(\mathbf{h} - \mathbf{a}) \cdot (\mathbf{c} - \mathbf{b}) = 0$$

Similarly, the altitude from vertex $\mathbf{b}$ is perpendicular to the opposite side $\mathbf{c} \mathbf{a}$,

$$(\mathbf{h} - \mathbf{b}) \cdot (\mathbf{a} - \mathbf{c}) = 0$$

Finally, the altitude from vertex $\mathbf{c}$ is perpendicular to the opposite side $\mathbf{a} \mathbf{b}$,

$$(\mathbf{h} - \mathbf{c}) \cdot (\mathbf{b} - \mathbf{a}) = 0$$

For the three foregoing equations, only two are linearly independent (the third follows from the other two).  So, to solve for $\mathbf{h}$ in terms of $\mathbf{a}$, $\mathbf{b}$, and $\mathbf{c}$, we typically parameterize the position of $\mathbf{h}$ as

$$\mathbf{h} = \mathbf{a} + s (\mathbf{b} - \mathbf{a}) + t (\mathbf{c} - \mathbf{a})$$

To find the orthocenter $\mathbf{h}$ and the ideal node position $\mathbf{e}$, we treat the triangle formed by a, b, and c as the base of a tetrahedron where e is the apex.

1. Solving for the Orthocenter parameters $s$ and $t$

We use the parametrization $h=a+s(b−a)+t(c−a)$ and substitute it into the orthogonality conditions. Let $u=b−a$ and $v=c−a$.

The conditions given are:

$$(h−b)⋅(a−c)=0$$

$$(h−c)⋅(b−a)=0$$

By substituting the parametrization into these equations, we get a system of two linear equations:

$$[su+(t−1)v]⋅u=0⟹s(u⋅u)+t(u⋅v)=u⋅v$$

$$[(s−1)u+tv]⋅v=0⟹s(u⋅v)+t(v⋅v)=u⋅v$$

Solving this system for $s$ and $t$:

$$s= 
(u⋅u)(v⋅v)−(u⋅v) 
2
 
(v⋅v−u⋅v)(u⋅v)$$
​	
 
$$t= 
(u⋅u)(v⋅v)−(u⋅v) 
2
 
(u⋅u−u⋅v)(u⋅v)
​$$
 
2. Finding $e$ from $h$

The ideal node $e$ is the point such that the vectors $(a−e)$, $(b−e)$, and $(c−e)$ are mutually orthogonal. Geometrically, if these three vectors are orthogonal, then $e$ must project directly onto the orthocenter $h$ of the opposite face △$abc$.

The vector $(e ∗ −h)$ is perpendicular to the plane $abc$. Therefore:

Direction: $e$ lies on a line passing through $h$ with a direction vector $n=(b−a)×(c−a)$.

Distance: The distance $L$ from $h$ to e ∗ is determined by the requirement that the interior angles at $e$ are $90^{\circ}$.

Using the Pythagorean theorem and the properties of orthogonal coordinates, the distance $L$ from $h$ to $e$ g ∗ is found by:

$$ L= −(h−a)⋅(h−b)$$
 
(Note: This dot product is negative because the vectors point away from the orthocenter toward the vertices in an acute triangle.)

Finally, the ideal position is:

$$e =h±L ∥n∥$$
 
(We choose the sign that places e on the correct side of the element face to maintain a positive Jacobian).
