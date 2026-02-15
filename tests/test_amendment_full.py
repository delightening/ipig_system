"""
å®Œæ•´ Amendmentï¼ˆè??«è??´ï?æµç??´å?æ¸¬è©¦

æ¸¬è©¦æµç?ï¼?
1. å»ºç? AUP è¨ˆç•«ä¸¦èµ°å®Œæ ¸?†æ?ç¨‹ï?ç°¡å??ˆï?
2. PI å»ºç? Amendment
3. PI ?äº¤ Amendment
4. IACUC_STAFF ?†é???Minor ???ªå? ADMIN_APPROVED
5. PI å»ºç?ç¬¬ä???Amendmentï¼ˆMajor è·¯ç?ï¼?
6. PI ?äº¤ ??IACUC_STAFF ?†é???Major ??CLASSIFIED
7. IACUC_STAFF ?‹å?å¯©æŸ¥ ??UNDER_REVIEW
8. å¯©æŸ¥å§”å“¡è¨˜é?æ±ºå? ???¨éƒ¨?¸å? ???ªå? APPROVED
9. é©—è??ˆæœ¬æ­·ç??‡ç??‹æ­·ç¨?
10. é©—è? Protocol amendments ?—è¡¨

?¨æ?ï¼?
    cd d:\\\\Coding\\\\ipig_system
    .venv\\Scripts\\python.exe tests/test_amendment_full.py
"""

import time
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from test_base import BaseApiTester, API_BASE_URL

# æ¸¬è©¦å¸³è?è¨­å?ï¼ˆæ²¿??AUP æ¸¬è©¦å¸³è?ï¼?
AMENDMENT_TEST_USERS = {
    "IACUC_STAFF": {"email": "staff_int_test@example.com", "password": "password123", "display_name": "IACUC Staff (?´å?æ¸¬è©¦)", "role_codes": ["IACUC_STAFF"]},
    "REVIEWER1":   {"email": "rev1_int_test@example.com",  "password": "password123", "display_name": "Reviewer 1 (?´å?æ¸¬è©¦)", "role_codes": ["REVIEWER"]},
    "REVIEWER2":   {"email": "rev2_int_test@example.com",  "password": "password123", "display_name": "Reviewer 2 (?´å?æ¸¬è©¦)", "role_codes": ["REVIEWER"]},
    "REVIEWER3":   {"email": "rev3_int_test@example.com",  "password": "password123", "display_name": "Reviewer 3 (?´å?æ¸¬è©¦)", "role_codes": ["REVIEWER"]},
    "IACUC_CHAIR": {"email": "chair_int_test@example.com", "password": "password123", "display_name": "IACUC Chair (?´å?æ¸¬è©¦)", "role_codes": ["REVIEWER", "IACUC_CHAIR"]},
    "PI":          {"email": "pi_int_test@example.com",    "password": "password123", "display_name": "PI (?´å?æ¸¬è©¦)",         "role_codes": ["PI"]},
    "VET":         {"email": "vet_int_test@example.com",   "password": "password123", "display_name": "VET (?´å?æ¸¬è©¦)",        "role_codes": ["VET"]},
}


def create_approved_protocol(t) -> str:
    """å»ºç?ä¸¦æ ¸?†ä???AUP è¨ˆç•«?¸ï?ç°¡å?æµç?ï¼‰ï??å‚³ protocol_id"""
    ts = int(time.time())

    # ?€å°å¯?¨è??«å…§å®?
    content = {
        "basic": {
            "study_title": f"Amendment æ¸¬è©¦?¨è??«æ›¸ {ts}",
            "project_code": f"AMD-TEST-{ts % 10000}",
            "project_type": "Research",
            "project_category": "Medicine",
            "is_glp": True,
            "apply_study_number": f"AMD-{ts % 1000000}",
            "registration_authorities": ["FDA"],
            "pi_user_id": t.user_ids["PI"],
            "pi": {"name": "PI", "phone": "0912-000-000", "email": "pi@test.com", "address": "?°å?å¸?},
            "sponsor": {"name": "æ¸¬è©¦?”ç©¶?ºé???, "contact_person": "?‹æ¸¬è©?, "contact_phone": "02-0000-0000", "contact_email": "s@test.com"},
            "facility": {"id": "FAC-001", "title": "ç¬¬ä??•ç‰©å¯¦é?ä¸­å?"},
            "housing_location": "B1"
        },
        "purpose": {
            "significance": "æ¸¬è©¦?¨é€”ã€?,
            "replacement": {"rationale": "?¡æ›¿ä»?–¹æ¡ˆã€?, "alt_search": {"platforms": ["PubMed"], "keywords": "test", "conclusion": "?¡ã€?}},
            "reduction": {"design": "?€å°æ¨£?¬ã€?},
            "duplicate": {"experiment": False}
        },
        "items": {"use_test_item": False},
        "design": {
            "procedures": "æ¯æ—¥è§€å¯Ÿã€?,
            "anesthesia": {"is_under_anesthesia": False},
            "pain": {"category": "B", "management_plan": "?¡ã€?},
            "endpoints": {"experimental_endpoint": "è¨ˆç•«çµæ???, "humane_endpoint": "é«”é?ä¸‹é? 20%??}
        },
        "guidelines": {"content": "?µå?è¦ç???},
        "animals": {
            "total_animals": 2,
            "animals": [{"species": "Pig", "strain": "L6", "sex": "MIXED", "number": 2,
                         "age_min": "8", "age_max": "10", "age_unlimited": False,
                         "weight_min": "20", "weight_max": "30", "weight_unlimited": False,
                         "housing_location": "B1"}]
        },
        "personnel": [{"name": "PI", "position": "?™æ?", "years_experience": "10",
                       "roles": ["è¨ˆç•«ä¸»æ?äº?], "trainings": ["?•ç‰©å¯¦é??«ç??¹è?"]}]
    }

    # 1. PI å»ºç?ä¸¦æ?äº?
    create_resp = t._req("POST", f"{API_BASE_URL}/protocols", role="PI", json={
        "title": f"Amendmentæ¸¬è©¦è¨ˆç•«_{ts}",
        "working_content": content,
        "start_date": "2026-05-01",
        "end_date": "2027-04-01",
        "pi_user_id": t.user_ids["PI"]
    })
    protocol_id = create_resp.json()["id"]

    # ?‡æ´¾ Co-editor
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/co-editors", role="IACUC_STAFF",
           json={"user_id": t.user_ids["IACUC_STAFF"], "protocol_id": protocol_id})

    # PI ?äº¤
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/submit", role="PI")

    # Staff ?å¯©
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_STAFF",
           json={"to_status": "PRE_REVIEW"})

    # ?²å…¥?«ç?å¯©æŸ¥
    reviewers = [t.user_ids["REVIEWER1"], t.user_ids["REVIEWER2"], t.user_ids["REVIEWER3"]]
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_STAFF",
           json={"to_status": "UNDER_REVIEW", "reviewer_ids": reviewers})

    # 3 ?å??¡ç?è¨€
    versions = t._req("GET", f"{API_BASE_URL}/protocols/{protocol_id}/versions", role="IACUC_STAFF").json()
    version_id = versions[0]["id"]
    for role in ["REVIEWER1", "REVIEWER2", "REVIEWER3"]:
        t._req("POST", f"{API_BASE_URL}/reviews/comments", role=role,
               json={"protocol_version_id": version_id, "content": f"OK from {role}"})

    # ä¸»å??¸å?
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_CHAIR",
           json={"to_status": "APPROVED", "remark": "?¸å???})

    return protocol_id


def run_amendment_test() -> bool:
    """?·è?å®Œæ•´ Amendment æ¸¬è©¦ï¼Œå??³æ˜¯?¦å…¨?¨é€šé?"""
    t = BaseApiTester("Amendment å®Œæ•´æµç?æ¸¬è©¦")

    # ========================================
    # ?ç½®ä½œæ¥­ï¼šå¸³?Ÿå»ºç«‹è??»å…¥
    # ========================================
    if not t.setup_test_users(AMENDMENT_TEST_USERS):
        return False
    if not t.login_all(AMENDMENT_TEST_USERS):
        return False

    # ========================================
    # Step 1: å»ºç?å·²æ ¸?†ç? AUP è¨ˆç•«??
    # ========================================
    t.step("å»ºç?å·²æ ¸?†ç? AUP è¨ˆç•«??)
    try:
        protocol_id = create_approved_protocol(t)
        t.record("å»ºç?å·²æ ¸?†è??«æ›¸", True, f"Protocol ID: {protocol_id[:8]}...")
    except Exception as e:
        t.record("å»ºç?å·²æ ¸?†è??«æ›¸", False, str(e))
        return t.print_summary()

    # ========================================
    # Step 2: PI å»ºç? Amendment (Minor è·¯ç?)
    # ========================================
    t.step("PI å»ºç? Amendmentï¼ˆMinor è·¯ç?ï¼?)
    ts = int(time.time())
    minor_resp = t._req("POST", f"{API_BASE_URL}/amendments", role="PI", json={
        "protocol_id": protocol_id,
        "title": f"å°è??´æ¸¬è©¦_{ts}",
        "description": "?´æ”¹å¯¦é?äººå“¡?¯çµ¡?¹å?",
        "change_items": ["personnel_contact"],
        "changes_content": {"personnel": {"contact_phone": "0912-111-111"}}
    })
    minor_amendment = minor_resp.json()
    minor_id = minor_amendment["id"]
    t.record("å»ºç? Minor Amendment", True, f"ID: {minor_id[:8]}..., status={minor_amendment['status']}")

    # ========================================
    # Step 3: PI ?äº¤ Minor Amendment
    # ========================================
    t.step("PI ?äº¤ Minor Amendment")
    submit_resp = t._req("POST", f"{API_BASE_URL}/amendments/{minor_id}/submit", role="PI")
    minor_status = submit_resp.json()["status"]
    t.record("?äº¤ Minor Amendment", minor_status in ("SUBMITTED", "submitted"),
             f"status={minor_status}")

    # ========================================
    # Step 4: IACUC_STAFF ?†é???Minor ???ªå? ADMIN_APPROVED
    # ========================================
    t.step("IACUC_STAFF ?†é???Minorï¼ˆè‡ª?•è??¿æ ¸?†ï?")
    classify_resp = t._req("POST", f"{API_BASE_URL}/amendments/{minor_id}/classify",
                            role="IACUC_STAFF", json={
                                "amendment_type": "MINOR",
                                "remark": "å°è??´ï?è¡Œæ”¿?¸å?"
                            })
    minor_final_status = classify_resp.json()["status"]
    t.record("Minor ?†é? ??ADMIN_APPROVED",
             minor_final_status in ("ADMIN_APPROVED", "admin_approved"),
             f"status={minor_final_status}")

    # ========================================
    # Step 5: PI å»ºç? Amendment (Major è·¯ç?)
    # ========================================
    t.step("PI å»ºç? Amendmentï¼ˆMajor è·¯ç?ï¼?)
    major_resp = t._req("POST", f"{API_BASE_URL}/amendments", role="PI", json={
        "protocol_id": protocol_id,
        "title": f"?å¤§è®Šæ›´æ¸¬è©¦_{ts}",
        "description": "å¢å?å¯¦é??•ç‰©?¸é?",
        "change_items": ["animal_count", "design"],
        "changes_content": {
            "animals": {"total_animals": 10, "reason": "çµ±è??†æ??€è¦æ›´å¤šæ¨£??},
            "design": {"procedures": "æ¯æ—¥è§€å¯?+ ?±é??æ¸¬??}
        }
    })
    major_amendment = major_resp.json()
    major_id = major_amendment["id"]
    t.record("å»ºç? Major Amendment", True, f"ID: {major_id[:8]}...")

    # ========================================
    # Step 6: PI ?äº¤ ??IACUC_STAFF ?†é???Major
    # ========================================
    t.step("PI ?äº¤ ??IACUC_STAFF ?†é???Major")
    t._req("POST", f"{API_BASE_URL}/amendments/{major_id}/submit", role="PI")

    classify_major_resp = t._req("POST", f"{API_BASE_URL}/amendments/{major_id}/classify",
                                  role="IACUC_STAFF", json={
                                      "amendment_type": "MAJOR",
                                      "remark": "?€è¦å??¡å¯©??
                                  })
    major_classified_status = classify_major_resp.json()["status"]
    t.record("Major ?†é? ??CLASSIFIED",
             major_classified_status in ("CLASSIFIED", "classified"),
             f"status={major_classified_status}")

    # ========================================
    # Step 7: é©—è?å¯©æŸ¥å§”å“¡?ªå??‡æ´¾ï¼ˆå??Ÿè??«è?è£½ï?
    # ========================================
    t.step("é©—è?å¯©æŸ¥å§”å“¡?ªå??‡æ´¾")
    assignments_resp = t._req("GET", f"{API_BASE_URL}/amendments/{major_id}/assignments",
                               role="IACUC_STAFF")
    assignments = assignments_resp.json()
    t.record("å¯©æŸ¥å§”å“¡?ªå??‡æ´¾", len(assignments) >= 2,
             f"??{len(assignments)} ä½å¯©?¥å???)

    # ========================================
    # Step 8: IACUC_STAFF ?‹å?å¯©æŸ¥ ??UNDER_REVIEW
    # ========================================
    t.step("IACUC_STAFF ?‹å?å¯©æŸ¥")
    review_resp = t._req("POST", f"{API_BASE_URL}/amendments/{major_id}/start-review",
                          role="IACUC_STAFF")
    review_status = review_resp.json()["status"]
    t.record("?‹å?å¯©æŸ¥ ??UNDER_REVIEW",
             review_status in ("UNDER_REVIEW", "under_review"),
             f"status={review_status}")

    # ========================================
    # Step 9: å¯©æŸ¥å§”å“¡è¨˜é?æ±ºå?ï¼ˆå…¨?¨æ ¸?????ªå? APPROVEDï¼?
    # ========================================
    t.step("å¯©æŸ¥å§”å“¡è¨˜é?æ±ºå?ï¼ˆå…¨?¨æ ¸?†ï?")

    # ?–å?å·²æ?æ´¾ç?å¯©æŸ¥å§”å“¡ ID
    reviewer_roles = []
    for a in assignments:
        reviewer_id = a["reviewer_id"]
        # ?¾å‡ºå°æ??„è??²å?ç¨?
        for role_name, uid in t.user_ids.items():
            if uid == reviewer_id:
                reviewer_roles.append(role_name)
                break

    decision_count = 0
    for role_name in reviewer_roles:
        try:
            t._req("POST", f"{API_BASE_URL}/amendments/{major_id}/decision",
                    role=role_name, json={
                        "decision": "APPROVE",
                        "comment": f"?Œæ?è®Šæ›´ ??by {role_name}"
                    })
            decision_count += 1
        except Exception as e:
            print(f"    ??{role_name} æ±ºå?å¤±æ?: {e}")

    t.record("å¯©æŸ¥å§”å“¡?¨éƒ¨?¸å?", decision_count == len(reviewer_roles),
             f"{decision_count}/{len(reviewer_roles)} ä½?)

    # é©—è??ªå??´æ–°??APPROVED
    final_resp = t._req("GET", f"{API_BASE_URL}/amendments/{major_id}", role="IACUC_STAFF")
    final_status = final_resp.json()["status"]
    t.record("?ªå??´æ–° ??APPROVED",
             final_status in ("APPROVED", "approved"),
             f"status={final_status}")

    # ========================================
    # Step 10: é©—è??ˆæœ¬æ­·ç?
    # ========================================
    t.step("é©—è??ˆæœ¬æ­·ç?")
    versions_resp = t._req("GET", f"{API_BASE_URL}/amendments/{major_id}/versions",
                            role="IACUC_STAFF")
    versions = versions_resp.json()
    t.record("Major ?ˆæœ¬æ­·ç?", len(versions) >= 1, f"??{len(versions)} ?‹ç???)

    # ========================================
    # Step 11: é©—è??€?‹æ­·ç¨?
    # ========================================
    t.step("é©—è??€?‹æ­·ç¨?)
    history_resp = t._req("GET", f"{API_BASE_URL}/amendments/{major_id}/history",
                           role="IACUC_STAFF")
    history = history_resp.json()
    # ?æ?æ­·ç?ï¼šDRAFT?’SUBMITTED?’CLASSIFIED?’UNDER_REVIEW?’APPROVED
    history_statuses = [h.get("to_status", "") for h in history]
    t.record("?€?‹æ­·ç¨‹å???,
             len(history) >= 4,
             f"??{len(history)} ç­? {' ??'.join(history_statuses[::-1])}")

    # ========================================
    # Step 12: é©—è? Protocol ??amendments ?—è¡¨
    # ========================================
    t.step("é©—è? Protocol amendments ?—è¡¨")
    proto_amendments_resp = t._req("GET", f"{API_BASE_URL}/protocols/{protocol_id}/amendments",
                                     role="IACUC_STAFF")
    proto_amendments = proto_amendments_resp.json()
    t.record("Protocol amendments ?—è¡¨",
             len(proto_amendments) >= 2,
             f"??{len(proto_amendments)} ??amendment")

    # ========================================
    # Step 13: é©—è? Amendment ?—è¡¨?¥è©¢ï¼ˆå«ç¯©é¸ï¼?
    # ========================================
    t.step("é©—è? Amendment ?—è¡¨?¥è©¢")
    all_resp = t._req("GET", f"{API_BASE_URL}/amendments", role="IACUC_STAFF")
    all_amendments = all_resp.json()
    t.record("Amendment ?—è¡¨?¥è©¢", len(all_amendments) >= 2,
             f"??{len(all_amendments)} ??)

    # ä¾ç??‹ç¯©??
    approved_resp = t._req("GET", f"{API_BASE_URL}/amendments?status=APPROVED",
                            role="IACUC_STAFF")
    approved_amendments = approved_resp.json()
    t.record("ä¾ç??‹ç¯©?¸ï?APPROVEDï¼?, len(approved_amendments) >= 1,
             f"??{len(approved_amendments)} ??)

    # ========================================
    # Step 14: é©—è?å¾…è??†æ•¸??
    # ========================================
    t.step("é©—è?å¾…è??†æ•¸??API")
    pending_resp = t._req("GET", f"{API_BASE_URL}/amendments/pending-count",
                           role="IACUC_STAFF")
    pending_data = pending_resp.json()
    t.record("å¾…è??†æ•¸??API", "count" in pending_data,
             f"pending count = {pending_data.get('count', 'N/A')}")

    # ========================================
    # å½™ç¸½
    # ========================================
    print(f"\n{'=' * 60}")
    print(f"[å®Œæ?] Amendment å®Œæ•´æµç?å®Œæ?ï¼?)
    print(f"  Protocol ID: {protocol_id}")
    print(f"  Minor Amendment ID: {minor_id} (ADMIN_APPROVED)")
    print(f"  Major Amendment ID: {major_id} (APPROVED)")
    print(f"{'=' * 60}")
    return t.print_summary()


if __name__ == "__main__":
    try:
        success = run_amendment_test()
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"\n[CRITICAL ERROR] Amendment æ¸¬è©¦å¤±æ?: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
