import { attributes } from "../../css/shared.js";
export const CONNECTION_STATUS_CLASS = "lt-connection-status";
export const CONNECTION_STATUS_BAR_CLASS = "lt-connection-status__bar";
/**
 * The attributes for the bar that says the connection is lost.
 *
 * By definition the server cannot tell you it is unreachable. When it is
 * down, nothing comes from it to render. So the component creates all the
 * content itself, and `data-ignore-morph` stops a later patch from tearing
 * it away the moment the connection is back.
 *
 * ```ts
 * <lt-connection-status {...lt.connectionStatus()} />
 * ```
 */
export const connectionStatus = ({ offlineText, onlineText, } = {}) => attributes({
    class: CONNECTION_STATUS_CLASS,
    "offline-text": offlineText,
    "online-text": onlineText,
    "data-ignore-morph": "",
});
//# sourceMappingURL=connection-status.js.map