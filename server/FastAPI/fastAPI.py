# FastAPI/fastAPI.py
from fastapi import FastAPI, HTTPException, Request
from pydantic import BaseModel
import httpx
import os
import logging

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("fastapi_logger")

RUST_HOST = os.getenv("OLLAMA_HOST", "http://rust-api:8080")  # Rust service URL

app = FastAPI()

class ChatRequest(BaseModel):
    prompt: str

@app.post("/chat")
async def chat(request: ChatRequest):
    logger.info(f"Received prompt: {request.prompt}")
    try:
        async with httpx.AsyncClient(timeout=60.0) as client:
            resp = await client.post(
                f"{RUST_HOST}/v1/run/tinyllama",
                json={"prompt": request.prompt}
            )
            logger.info(f"Rust API response status: {resp.status_code}")
            resp.raise_for_status()
            data = resp.json()
            logger.info(f"Rust API response data: {data}")
            return {"response": data}
    except httpx.HTTPStatusError as e:
        logger.error(f"HTTPStatusError: {e.response.status_code} - {e.response.text}")
        raise HTTPException(status_code=e.response.status_code, detail=e.response.text)
    except httpx.RequestError as e:
        logger.error(f"RequestError: {str(e)}")
        raise HTTPException(status_code=500, detail=f"Request error: {str(e)}")
    except Exception as e:
        logger.exception("Unexpected error")
        raise HTTPException(status_code=500, detail=str(e))
