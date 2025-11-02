fn configure(meta) {
    // budgets scale with difficulty + enemy density
    let dens = meta["enemy_count"] / (meta["width"] * meta["height"]/50).max(1);
    let diff = meta["difficulty"];

    let traps = (1 + diff/2 + dens).min(5);
    let terrain_edits = (2 + diff + dens).min(6);
    let spawns = (1 + diff + dens/2).min(5);

    let hints = #{
        "cover_density": 0.5 + diff * 0.1,
        "preferred_chokes": [ "bridge_west", "arch_north" ],
        "forbid_ops": [ "lava_flood" ]
    };

    return #{
        "traps": traps,
        "terrain_edits": terrain_edits,
        "spawns": spawns,
        "hints": hints
    };
}
