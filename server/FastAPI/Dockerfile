FROM python:3.12-slim

WORKDIR /app

RUN pip install --default-timeout=3000 --no-cache-dir fastapi uvicorn httpx better-profanity "llm-guard[all]"

COPY *.py /app/

EXPOSE 8000

CMD ["uvicorn", "fastAPI:app", "--host", "0.0.0.0", "--port", "8000"]
