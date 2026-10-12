# WebKit popover target interface guard

Inspected 2026-10-12, revision `73aa6c89e2cb77c46184a81aec944e4ab99d114d`.

Source: https://raw.githubusercontent.com/WebKit/WebKit/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/html/HTMLFormControlElement.cpp

Original SHA-256: `84bb969f25feec272fb4b764e80c13cdaf9bbbe0a4ee811ef5872a55cc498740`.

Original source line 348 in HTMLFormControlElement::popoverTargetElement downcasts the already resolved associated Element to HTMLElement before inspecting popover state; a failed cast returns null. This supports a foreign-target rejection after association, without searching for another matching ID. The witness is scoped to that guard, not other WebKit behavior.

This is metadata and behavioral summary only, with no source-code excerpt. The complete original is private research beside this file and carries GNU Library GPL notices; its redistribution or implementation ingestion is not proposed. The source catalog retains this original URL/hash identity as metadata only.
