export const elements = {
    "fs-field": {
        "link": "https://fristil.sobernetics.no/components/field/",
        "attributes": {
            "invalid": {
                "type": "flag"
            },
            "disabled": {
                "type": "flag"
            },
            "optional": {
                "type": "flag"
            },
            "required-marker": {
                "type": "values",
                "values": [
                    "symbol",
                    "text",
                    "none"
                ]
            },
            "control-id": {
                "type": "text"
            },
            "described-by": {
                "type": "text"
            }
        }
    },
    "fs-tabs": {
        "link": "https://fristil.sobernetics.no/components/tabs/",
        "attributes": {
            "server-controlled": {
                "type": "flag"
            }
        }
    },
    "fs-error-summary": {
        "link": "https://fristil.sobernetics.no/components/error-summary/",
        "attributes": {
            "data-autofocus": {
                "type": "values",
                "values": [
                    "false"
                ]
            },
            "hidden": {
                "type": "flag"
            }
        }
    },
    "fs-popover": {
        "link": "https://fristil.sobernetics.no/components/popover/",
        "attributes": {
            "open": {
                "type": "flag"
            },
            "placement": {
                "type": "values",
                "values": [
                    "bottom-start",
                    "bottom-end",
                    "top-start",
                    "top-end"
                ]
            },
            "server-controlled": {
                "type": "flag"
            }
        }
    },
    "fs-suggestion": {
        "link": "https://fristil.sobernetics.no/components/suggestion/",
        "attributes": {
            "prefiltered": {
                "type": "flag"
            },
            "count-none": {
                "type": "text"
            },
            "count-zero": {
                "type": "text"
            },
            "count-one": {
                "type": "text"
            },
            "count-two": {
                "type": "text"
            },
            "count-few": {
                "type": "text"
            },
            "count-many": {
                "type": "text"
            },
            "count-other": {
                "type": "text"
            },
            "server-controlled": {
                "type": "flag"
            }
        }
    },
    "fs-dialog": {
        "link": "https://fristil.sobernetics.no/components/dialog/",
        "attributes": {
            "open": {
                "type": "flag"
            },
            "server-controlled": {
                "type": "flag"
            }
        }
    },
    "fs-toast": {
        "link": "https://fristil.sobernetics.no/components/toast/",
        "attributes": {
            "duration": {
                "type": "number"
            },
            "label": {
                "type": "text"
            },
            "close-label": {
                "type": "text"
            }
        }
    },
    "fs-session-timeout": {
        "link": "https://fristil.sobernetics.no/components/session-timeout/",
        "attributes": {
            "warn-at": {
                "type": "number"
            },
            "expires-at": {
                "type": "number"
            }
        }
    },
    "fs-connection-status": {
        "link": "https://fristil.sobernetics.no/components/connection-status/",
        "attributes": {
            "offline-text": {
                "type": "text"
            },
            "online-text": {
                "type": "text"
            }
        }
    }
};
//# sourceMappingURL=elements.js.map