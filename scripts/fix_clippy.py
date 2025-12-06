import re  
  
with open('astraweave-net-ecs/src/lib.rs', 'r', encoding='utf-8') as f:  
    content = f.read()  
  
content = re.sub(r"while let Some\(\(entity, client\)\) = q\.next\(\) \{", "for (entity, client) in q {", content)  
content = re.sub(r"while let Some\(\(entity, authority\)\) = q\.next\(\) \{", "for (entity, authority) in q {", content)  
content = re.sub(r"for \(_client_id, sender\) in ^&authority\.connected_clients \{", "for sender in authority.connected_clients.values() {", content)  
  
with open('astraweave-net-ecs/src/lib.rs', 'w', encoding='utf-8') as f:  
    f.write(content)  
  
print("Fixed all clippy issues") 
