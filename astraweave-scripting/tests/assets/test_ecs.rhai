let pos = vec3(10.0, 0.0, 10.0);
commands.spawn_prefab("enemy_grunt", pos);
commands.set_position(entity_id, vec3(5.0, 5.0, 5.0));
log("ECS commands issued");
