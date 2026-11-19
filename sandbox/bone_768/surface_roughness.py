"""Boundary-surface roughness of a hex mesh: angles between normals of
adjacent boundary quads (sharing an edge).  Smaller is smoother."""
import sys, numpy as np, meshio
from collections import defaultdict
FACES = [(0,3,2,1),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7)]
def stats(path):
    m = meshio.read(path); pts = m.points; hexes = m.cells_dict["hexahedron"]
    count = defaultdict(int); orient = {}
    for h in hexes:
        for f in FACES:
            q = tuple(h[i] for i in f); k = tuple(sorted(q)); count[k] += 1; orient[k] = q
    quads = [orient[k] for k, c in count.items() if c == 1]
    Q = np.array(quads)
    n = np.cross(pts[Q[:,2]] - pts[Q[:,0]], pts[Q[:,3]] - pts[Q[:,1]])
    n /= np.linalg.norm(n, axis=1)[:, None]
    edges = defaultdict(list)
    for i, q in enumerate(quads):
        for a in range(4):
            edges[tuple(sorted((q[a], q[(a+1)%4])))].append(i)
    ang = np.array([np.degrees(np.arccos(np.clip(np.dot(n[i], n[j]), -1, 1)))
                    for v in edges.values() if len(v) == 2 for i, j in [v]])
    return len(quads), ang
for label, path in zip(sys.argv[1::2], sys.argv[2::2]):
    nq, a = stats(path)
    print(f"{label:22} quads {nq:5d}  edges {len(a):5d}  median {np.median(a):5.2f}  p90 {np.percentile(a,90):5.2f}  p99 {np.percentile(a,99):5.2f}  max {a.max():6.2f}  >20deg {np.sum(a>20):4d}")
