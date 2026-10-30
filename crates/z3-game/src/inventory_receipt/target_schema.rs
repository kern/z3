// Ordered item grants: exactly the original76 rows, including aliases and signed values.
target_fields! {
    Sword => 0xf359;
    Shield => 0xf35a;
    FireRod => 0xf345;
    IceRod => 0xf346;
    Hammer => 0xf34b;
    Hookshot => 0xf342;
    Bow => 0xf340;
    Boomerang => 0xf341;
    Mushroom => 0xf344;
    FirstBottle => 0xf35c;
    Bombos => 0xf347;
    Ether => 0xf348;
    Quake => 0xf349;
    Lamp => 0xf34a;
    Flute => 0xf34c;
    Somaria => 0xf350;
    HeartPieces => 0xf36b;
    Byrna => 0xf351;
    Cape => 0xf352;
    Mirror => 0xf353;
    Gloves => 0xf354;
    Book => 0xf34e;
    Flippers => 0xf356;
    MoonPearl => 0xf357;
    Crystals => 0xf37a;
    Net => 0xf34d;
    Armor => 0xf35b;
    Keys => 0xf36f;
    Compass => 0xf364;
    MaxHealth => 0xf36c;
    BombRefill => 0xf375;
    Health => 0xf36d;
    Magic => 0xf36e;
    BigKey => 0xf366;
    DungeonMap => 0xf368;
    RupeesGoal => 0xf360;
    Pendants => 0xf374;
    HeartRefill => 0xf372;
    ArrowRefill => 0xf376;
    MagicRefill => 0xf373;
    Boots => 0xf355;
}

const RECEIPTS: [ReceiptSpec; 76] = [
    ReceiptSpec {
        target: InventoryTarget::Sword,
        initial_value: 1,
    }, // 00
    ReceiptSpec {
        target: InventoryTarget::Sword,
        initial_value: 2,
    }, // 01
    ReceiptSpec {
        target: InventoryTarget::Sword,
        initial_value: 3,
    }, // 02
    ReceiptSpec {
        target: InventoryTarget::Sword,
        initial_value: 4,
    }, // 03
    ReceiptSpec {
        target: InventoryTarget::Shield,
        initial_value: 1,
    }, // 04
    ReceiptSpec {
        target: InventoryTarget::Shield,
        initial_value: 2,
    }, // 05
    ReceiptSpec {
        target: InventoryTarget::Shield,
        initial_value: 3,
    }, // 06
    ReceiptSpec {
        target: InventoryTarget::FireRod,
        initial_value: 1,
    }, // 07
    ReceiptSpec {
        target: InventoryTarget::IceRod,
        initial_value: 1,
    }, // 08
    ReceiptSpec {
        target: InventoryTarget::Hammer,
        initial_value: 1,
    }, // 09
    ReceiptSpec {
        target: InventoryTarget::Hookshot,
        initial_value: 1,
    }, // 0a
    ReceiptSpec {
        target: InventoryTarget::Bow,
        initial_value: 1,
    }, // 0b
    ReceiptSpec {
        target: InventoryTarget::Boomerang,
        initial_value: 1,
    }, // 0c
    ReceiptSpec {
        target: InventoryTarget::Mushroom,
        initial_value: 2,
    }, // 0d
    ReceiptSpec {
        target: InventoryTarget::FirstBottle,
        initial_value: -1,
    }, // 0e
    ReceiptSpec {
        target: InventoryTarget::Bombos,
        initial_value: 1,
    }, // 0f
    ReceiptSpec {
        target: InventoryTarget::Ether,
        initial_value: 1,
    }, // 10
    ReceiptSpec {
        target: InventoryTarget::Quake,
        initial_value: 1,
    }, // 11
    ReceiptSpec {
        target: InventoryTarget::Lamp,
        initial_value: 1,
    }, // 12
    ReceiptSpec {
        target: InventoryTarget::Flute,
        initial_value: 1,
    }, // 13
    ReceiptSpec {
        target: InventoryTarget::Flute,
        initial_value: 2,
    }, // 14
    ReceiptSpec {
        target: InventoryTarget::Somaria,
        initial_value: 1,
    }, // 15
    ReceiptSpec {
        target: InventoryTarget::FirstBottle,
        initial_value: -1,
    }, // 16
    ReceiptSpec {
        target: InventoryTarget::HeartPieces,
        initial_value: -1,
    }, // 17
    ReceiptSpec {
        target: InventoryTarget::Byrna,
        initial_value: 1,
    }, // 18
    ReceiptSpec {
        target: InventoryTarget::Cape,
        initial_value: 1,
    }, // 19
    ReceiptSpec {
        target: InventoryTarget::Mirror,
        initial_value: 2,
    }, // 1a
    ReceiptSpec {
        target: InventoryTarget::Gloves,
        initial_value: 1,
    }, // 1b
    ReceiptSpec {
        target: InventoryTarget::Gloves,
        initial_value: 2,
    }, // 1c
    ReceiptSpec {
        target: InventoryTarget::Book,
        initial_value: 1,
    }, // 1d
    ReceiptSpec {
        target: InventoryTarget::Flippers,
        initial_value: 1,
    }, // 1e
    ReceiptSpec {
        target: InventoryTarget::MoonPearl,
        initial_value: 1,
    }, // 1f
    ReceiptSpec {
        target: InventoryTarget::Crystals,
        initial_value: -1,
    }, // 20
    ReceiptSpec {
        target: InventoryTarget::Net,
        initial_value: 1,
    }, // 21
    ReceiptSpec {
        target: InventoryTarget::Armor,
        initial_value: -1,
    }, // 22
    ReceiptSpec {
        target: InventoryTarget::Armor,
        initial_value: 2,
    }, // 23
    ReceiptSpec {
        target: InventoryTarget::Keys,
        initial_value: -1,
    }, // 24
    ReceiptSpec {
        target: InventoryTarget::Compass,
        initial_value: -1,
    }, // 25
    ReceiptSpec {
        target: InventoryTarget::MaxHealth,
        initial_value: -1,
    }, // 26
    ReceiptSpec {
        target: InventoryTarget::BombRefill,
        initial_value: -1,
    }, // 27
    ReceiptSpec {
        target: InventoryTarget::BombRefill,
        initial_value: -1,
    }, // 28
    ReceiptSpec {
        target: InventoryTarget::Mushroom,
        initial_value: -1,
    }, // 29
    ReceiptSpec {
        target: InventoryTarget::Boomerang,
        initial_value: 2,
    }, // 2a
    ReceiptSpec {
        target: InventoryTarget::FirstBottle,
        initial_value: -1,
    }, // 2b
    ReceiptSpec {
        target: InventoryTarget::FirstBottle,
        initial_value: -1,
    }, // 2c
    ReceiptSpec {
        target: InventoryTarget::FirstBottle,
        initial_value: -1,
    }, // 2d
    ReceiptSpec {
        target: InventoryTarget::Health,
        initial_value: -1,
    }, // 2e
    ReceiptSpec {
        target: InventoryTarget::Magic,
        initial_value: -1,
    }, // 2f
    ReceiptSpec {
        target: InventoryTarget::Magic,
        initial_value: -1,
    }, // 30
    ReceiptSpec {
        target: InventoryTarget::BombRefill,
        initial_value: -1,
    }, // 31
    ReceiptSpec {
        target: InventoryTarget::BigKey,
        initial_value: -1,
    }, // 32
    ReceiptSpec {
        target: InventoryTarget::DungeonMap,
        initial_value: -1,
    }, // 33
    ReceiptSpec {
        target: InventoryTarget::RupeesGoal,
        initial_value: -1,
    }, // 34
    ReceiptSpec {
        target: InventoryTarget::RupeesGoal,
        initial_value: -5,
    }, // 35
    ReceiptSpec {
        target: InventoryTarget::RupeesGoal,
        initial_value: -20,
    }, // 36
    ReceiptSpec {
        target: InventoryTarget::Pendants,
        initial_value: -1,
    }, // 37
    ReceiptSpec {
        target: InventoryTarget::Pendants,
        initial_value: -1,
    }, // 38
    ReceiptSpec {
        target: InventoryTarget::Pendants,
        initial_value: -1,
    }, // 39
    ReceiptSpec {
        target: InventoryTarget::Bow,
        initial_value: 1,
    }, // 3a
    ReceiptSpec {
        target: InventoryTarget::Bow,
        initial_value: 3,
    }, // 3b
    ReceiptSpec {
        target: InventoryTarget::FirstBottle,
        initial_value: -1,
    }, // 3c
    ReceiptSpec {
        target: InventoryTarget::FirstBottle,
        initial_value: -1,
    }, // 3d
    ReceiptSpec {
        target: InventoryTarget::MaxHealth,
        initial_value: -1,
    }, // 3e
    ReceiptSpec {
        target: InventoryTarget::MaxHealth,
        initial_value: -1,
    }, // 3f
    ReceiptSpec {
        target: InventoryTarget::RupeesGoal,
        initial_value: -100,
    }, // 40
    ReceiptSpec {
        target: InventoryTarget::RupeesGoal,
        initial_value: -50,
    }, // 41
    ReceiptSpec {
        target: InventoryTarget::HeartRefill,
        initial_value: -1,
    }, // 42
    ReceiptSpec {
        target: InventoryTarget::ArrowRefill,
        initial_value: 1,
    }, // 43
    ReceiptSpec {
        target: InventoryTarget::ArrowRefill,
        initial_value: 10,
    }, // 44
    ReceiptSpec {
        target: InventoryTarget::MagicRefill,
        initial_value: -1,
    }, // 45
    ReceiptSpec {
        target: InventoryTarget::RupeesGoal,
        initial_value: -1,
    }, // 46
    ReceiptSpec {
        target: InventoryTarget::RupeesGoal,
        initial_value: -1,
    }, // 47
    ReceiptSpec {
        target: InventoryTarget::FirstBottle,
        initial_value: -1,
    }, // 48
    ReceiptSpec {
        target: InventoryTarget::Sword,
        initial_value: 1,
    }, // 49
    ReceiptSpec {
        target: InventoryTarget::Flute,
        initial_value: 3,
    }, // 4a
    ReceiptSpec {
        target: InventoryTarget::Boots,
        initial_value: 1,
    }, // 4b
];
