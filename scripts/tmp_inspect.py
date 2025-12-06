import sys
p = 'assets/models/character-a.glb'
with open(p,'rb') as f:
    d = f.read()
print('len', len(d))
print('has uri', b'"uri"' in d)
print('has png', b'.png' in d)
print('has ktx2', b'ktx2' in d)
