from transformers import CLIPProcessor, CLIPModel
from PIL import Image
import torch
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

TAXONOMY = {
    "Persona_Sola": "A photo of a single person, portrait, selfie",
    "Grupo_Personas": "A photo of a group of people, gathering, celebration",
    "Documentos_Texto": "A photo of a document, receipt, text, scan, screenshot",
    "Paisajes_Naturaleza": "A photo of a natural landscape, outdoors, nature",
    "Vehiculos_Motor": "A photo of a car, motorcycle, truck, vehicle",
    "Mascotas_Animales": "A photo of a dog, cat, pet, animal",
    "Comida_Bebida": "A photo of food, meal, drink",
    "Arquitectura_Espacios": "A photo of a building, architecture, indoor room",
    "Arte_Graficos": "Digital art, meme, vector graphic, screenshot of software",
}

class CLIPClassifier:
    def __init__(self, model_name="openai/clip-vit-base-patch32"):
        self.device = "cuda" if torch.cuda.is_available() else "cpu"
        logger.info(f"Loading CLIP model on {self.device}...")
        self.model = CLIPModel.from_pretrained(model_name).to(self.device)
        self.processor = CLIPProcessor.from_pretrained(model_name)
        self.categories = list(TAXONOMY.keys())
        self.prompts = list(TAXONOMY.values())
        logger.info("CLIP model loaded.")

    def classify_batch(self, image_paths):
        images = []
        valid_paths = []
        for path in image_paths:
            try:
                img = Image.open(path).convert("RGB")
                # Resize to max 512px to save VRAM and bus bandwidth
                img.thumbnail((512, 512), Image.Resampling.LANCZOS)
                images.append(img)
                valid_paths.append(path)
            except Exception as e:
                logger.error(f"Error loading image {path}: {e}")
                
        if not images:
            return []

        inputs = self.processor(text=self.prompts, images=images, return_tensors="pt", padding=True, truncation=True).to(self.device)
        
        with torch.no_grad():
            outputs = self.model(**inputs)
            logits_per_image = outputs.logits_per_image
            probs = logits_per_image.softmax(dim=1)

        results = []
        for i, path in enumerate(valid_paths):
            path_probs = probs[i].cpu().tolist()
            max_idx = path_probs.index(max(path_probs))
            results.append({
                "path": path,
                "category": self.categories[max_idx],
                "confidence": path_probs[max_idx]
            })

        return results

classifier_instance = None

def get_classifier():
    global classifier_instance
    if classifier_instance is None:
        classifier_instance = CLIPClassifier()
    return classifier_instance
