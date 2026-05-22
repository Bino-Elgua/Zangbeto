# Zàngbétò v1.0 — Immune + Shrine Orchestration
# Makefile (Top Level)

SHELL := /bin/bash

# Paths
IMMUNE := immune
SHRINE := shrine
SHARED := shared
REC_OUT := $(IMMUNE)/receipts/out

# Env
include .env
export

.PHONY: deps patrol submit shrine-bootstrap sabbath clean validate-schema test-emission

deps:
	@echo "Installing deps..."
	@pip install -r requirements.txt || true
	@cd $(SHRINE) && npm i

patrol:
	@echo "Running veils under sandbox limits..."
	@echo "Veils running via steward-heartbeat and native steward..."

submit:
	@echo "Submitting receipts on-chain..."
	@node $(SHRINE)/scripts/submit_onchain_receipt.js $$PKG_ID $$WSET_ID $$LEDGER_ID $$REG_ID $$STATS_ID

shrine-bootstrap:
	@echo "Publishing Move pkg and initializing objects..."
	@cd $(SHRINE) && ./scripts/bootstrap.sh

sabbath:
	@echo "Weekly Sabbath seal routine..."
	@echo "Run ops/sabbath_checklist.md manually to seal the week."

validate-schema:
	@echo "Validating diagnostic JSON schema..."
	@cd crates/omo-diagnostic && cargo check

test-emission:
	@echo "Testing diagnostic emission..."
	@python3 omo_diagnostic.py --package test --file test.py --line 10 --code "OMO-ERR-001" --severity error --message "Test diagnostic" --repair-id "fix-test"

clean:
	@rm -rf $(REC_OUT)/*.json
