from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import List
from classifier import get_classifier

app = FastAPI(title="orDB AI Microservice")

model_ready = False

class ClassifyRequest(BaseModel):
    images: List[str]

class ClassifyResult(BaseModel):
    path: str
    category: str
    confidence: float

class BatchClassifyResponse(BaseModel):
    results: List[ClassifyResult]

@app.on_event("startup")
async def startup_event():
    global model_ready
    get_classifier()
    model_ready = True

@app.get("/health")
def health_check():
    if model_ready:
        return {"status": "ready"}
    return {"status": "loading"}

@app.post("/classify/batch", response_model=BatchClassifyResponse)
def classify_batch(req: ClassifyRequest):
    classifier = get_classifier()
    try:
        results = classifier.classify_batch(req.images)
        return {"results": results}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))
