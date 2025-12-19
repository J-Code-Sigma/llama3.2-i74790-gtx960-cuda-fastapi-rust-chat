# FastAPI/fastAPI.py
from fastapi import FastAPI, HTTPException, Request
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import httpx
import os
import logging

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("fastapi_logger")

RUST_HOST = os.getenv("OLLAMA_HOST", "http://rust-api:8080")  # Rust service URL

from filter import is_harmful
from llm_guard.input_scanners import PromptInjection, Toxicity

# Initialize scanners (this might take some time on the first run as models download)
# We handle initialization globally to avoid reloading models on every request.
try:
    scanners = [PromptInjection(), Toxicity()]
except Exception as e:
    logger.error(f"Failed to initialize LLM-Guard scanners: {e}")
    scanners = []

app = FastAPI()

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

class ChatRequest(BaseModel):
    prompt: str

@app.post("/chat")
async def chat(request: ChatRequest):
    logger.info(f"Received prompt: {request.prompt}")
    
    # Layer 1: Keyword/Profanity Filter (Fast)
    if is_harmful(request.prompt):
        logger.warning(f"Blocked by keyword filter: {request.prompt}")
        raise HTTPException(status_code=400, detail="The prompt contains content that is not allowed.")
    
    # Layer 2: LLM-Guard Scanners (Intent/Injection)
    for scanner in scanners:
        try:
            _, is_valid, risk_score = scanner.scan(request.prompt)
        except Exception as e:
            logger.error(f"Error during scanning with {scanner.__class__.__name__}: {e}")
            # In case of other errors in scanner, we let it pass to the next layer
            continue
            
        if not is_valid:
            logger.warning(f"Blocked by {scanner.__class__.__name__} (risk: {risk_score}): {request.prompt}")
            raise HTTPException(status_code=400, detail=f"Request blocked for safety reasons ({scanner.__class__.__name__}).")
    try:
        async with httpx.AsyncClient(timeout=120.0) as client:
            resp = await client.post(
                f"{RUST_HOST}/v1/chat",
                json={"prompt": request.prompt}
            )
            logger.info(f"Rust API response status: {resp.status_code}")
            resp.raise_for_status()
            data = resp.json()
            logger.info(f"Rust API response data: {data}")
            return data
    except HTTPException:
        # Re-raise HTTPExceptions (like the 400s from scanners) so they reach the client
        raise
    except httpx.HTTPStatusError as e:
        logger.error(f"HTTPStatusError: {e.response.status_code} - {e.response.text}")
        raise HTTPException(status_code=e.response.status_code, detail=e.response.text)
    except httpx.RequestError as e:
        logger.error(f"RequestError: {str(e)}")
        raise HTTPException(status_code=500, detail=f"Request error: {str(e)}")
    except Exception as e:
        logger.exception("Unexpected error")
        raise HTTPException(status_code=500, detail=str(e))
