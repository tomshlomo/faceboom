
import sys
sys.path.append('/Users/tomshlomo/VSCodeProjects/pyo3games/mp_env/lib/python3.9/site-packages')
# print("sys path len:", len(sys.path))

import cv2
import mediapipe as mp

cap = cv2.VideoCapture(0)

face_mesh = mp.solutions.face_mesh.FaceMesh(
    max_num_faces=1,
    refine_landmarks=True,
    min_detection_confidence=0.5,
    min_tracking_confidence=0.5,
)

def foo(x):
    print(sys.path)
    return x * 2

def get_coords(x):
    success, image = cap.read()
    if not success:
        return 0
    results = face_mesh.process(cv2.cvtColor(image, cv2.COLOR_BGR2RGB))
    return results.multi_face_landmarks[0].landmark[0].x, results.multi_face_landmarks[0].landmark[0].y
    
def get_coords_from_img(image):
    # success, image = cap.read()
    results = face_mesh.process(cv2.cvtColor(image, cv2.COLOR_BGR2RGB))
    return (results.multi_face_landmarks[0].landmark[0].x, results.multi_face_landmarks[0].landmark[0].y)