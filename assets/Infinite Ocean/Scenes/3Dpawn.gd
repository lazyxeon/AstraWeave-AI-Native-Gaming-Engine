extends MeshInstance3D

var direction : Vector3 = Vector3.ZERO;
var rotate : float = 0.0;
var speed : float = 0.2;

@onready var ocean = $"../Ocean"

func inputMove():
	direction = Vector3.ZERO;
	rotate = 0.0;
	speed = 0.2;
	if Input.is_action_pressed("Strafe Right"):
		direction.x += 1;
	if Input.is_action_pressed("Strafe Left"):
		direction.x -= 1;
	if Input.is_action_pressed("Backward"):
		direction.z += 1;
	if Input.is_action_pressed("Forward"):
		direction.z -= 1;
	if direction != Vector3.ZERO:
		direction = direction.normalized();
	if Input.is_action_pressed("Rotate Right"):
		rotate = -1.0;
	if Input.is_action_pressed("Rotate Left"):
		rotate = 1.0;
	if Input.is_action_pressed("Speed"):
		speed = 1.0;

# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	inputMove();
	# get the forward and right directions
	var forward = global_transform.basis.z;
	var right = global_transform.basis.x;
	
	# Adjust movement direction by rotation transform
	var relativeDir = (forward * direction.z + right * direction.x);
	
	self.position += relativeDir * speed;
	self.rotation.y += rotate * 0.1;
	ocean.position = self.position - Vector3(0.0,1.0,0.0);
