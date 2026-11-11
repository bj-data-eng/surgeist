# Web IDL DOMException source excerpt

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). This excerpt retains the
IDL and names table from the [pinned Web IDL source](https://github.com/whatwg/webidl/blob/0620cf171d1b97138847088748e363cae4005156/index.bs) at revision
`0620cf171d1b97138847088748e363cae4005156`. Specification text is licensed under
[Creative Commons Attribution 4.0 International](https://creativecommons.org/licenses/by/4.0/).
See the [retained WHATWG license](../licenses/whatwg/LICENSE) for code use.

The complete source file has SHA-256
`8db18145ddf051a30d6cad35fde25537fad8c9b0ee2f993e341091cd51c31013`.

The [custom JavaScript binding](https://github.com/whatwg/webidl/blob/0620cf171d1b97138847088748e363cae4005156/index.bs#L14882)
specifies the prototype's Error slots. The [creation steps](https://github.com/whatwg/webidl/blob/0620cf171d1b97138847088748e363cae4005156/index.bs#L14919)
specify native exception initialization in the current realm.

## Interface declaration

<a id="idl-DOMException"></a>

```webidl
[Exposed=*,
 Serializable]
interface DOMException { // but see below note about JavaScript binding
  constructor(optional DOMString message = "", optional DOMString name = "Error");
  readonly attribute DOMString name;
  readonly attribute DOMString message;
  readonly attribute unsigned short code;

  const unsigned short INDEX_SIZE_ERR = 1;
  const unsigned short DOMSTRING_SIZE_ERR = 2;
  const unsigned short HIERARCHY_REQUEST_ERR = 3;
  const unsigned short WRONG_DOCUMENT_ERR = 4;
  const unsigned short INVALID_CHARACTER_ERR = 5;
  const unsigned short NO_DATA_ALLOWED_ERR = 6;
  const unsigned short NO_MODIFICATION_ALLOWED_ERR = 7;
  const unsigned short NOT_FOUND_ERR = 8;
  const unsigned short NOT_SUPPORTED_ERR = 9;
  const unsigned short INUSE_ATTRIBUTE_ERR = 10;
  const unsigned short INVALID_STATE_ERR = 11;
  const unsigned short SYNTAX_ERR = 12;
  const unsigned short INVALID_MODIFICATION_ERR = 13;
  const unsigned short NAMESPACE_ERR = 14;
  const unsigned short INVALID_ACCESS_ERR = 15;
  const unsigned short VALIDATION_ERR = 16;
  const unsigned short TYPE_MISMATCH_ERR = 17;
  const unsigned short SECURITY_ERR = 18;
  const unsigned short NETWORK_ERR = 19;
  const unsigned short ABORT_ERR = 20;
  const unsigned short URL_MISMATCH_ERR = 21;
  const unsigned short QUOTA_EXCEEDED_ERR = 22;
  const unsigned short TIMEOUT_ERR = 23;
  const unsigned short INVALID_NODE_TYPE_ERR = 24;
  const unsigned short DATA_CLONE_ERR = 25;
};
```

## Names table

```html
<table id="error-names" class="vert data">
    <thead>
        <tr><th>Name</th><th>Description</th><th>Legacy code name and value</th></tr>
    </thead>
    <tbody>
        <tr class="deprecated">
            <td>"<dfn id="indexsizeerror" exception export><code>IndexSizeError</code></dfn>"</td>
            <td><strong>Deprecated.</strong> Use {{RangeError}} instead.</td>
            <td><dfn id="dom-domexception-index_size_err" for="DOMException" const export><code>INDEX_SIZE_ERR</code></dfn>&nbsp;(1)</td>
        </tr>
        <tr>
            <td>"<dfn id="hierarchyrequesterror" exception export><code>HierarchyRequestError</code></dfn>"</td>
            <td>The operation would yield an incorrect [=node tree=]. [[DOM]]</td>
            <td><dfn id="dom-domexception-hierarchy_request_err" for="DOMException" const export><code>HIERARCHY_REQUEST_ERR</code></dfn>&nbsp;(3)</td>
        </tr>
        <tr>
            <td>"<dfn id="wrongdocumenterror" exception export><code>WrongDocumentError</code></dfn>"</td>
            <td>The object is in the wrong [=/document=]. [[DOM]]</td>
            <td><dfn id="dom-domexception-wrong_document_err" for="DOMException" const export><code>WRONG_DOCUMENT_ERR</code></dfn>&nbsp;(4)</td>
        </tr>
        <tr>
            <td>"<dfn id="invalidcharactererror" exception export><code>InvalidCharacterError</code></dfn>"</td>
            <td>The string contains invalid characters.</td>
            <td><dfn id="dom-domexception-invalid_character_err" for="DOMException" const export><code>INVALID_CHARACTER_ERR</code></dfn>&nbsp;(5)</td>
        </tr>
        <tr>
            <td>"<dfn id="nomodificationallowederror" exception export><code>NoModificationAllowedError</code></dfn>"</td>
            <td>The object can not be modified.</td>
            <td><dfn id="dom-domexception-no_modification_allowed_err" for="DOMException" const export><code>NO_MODIFICATION_ALLOWED_ERR</code></dfn>&nbsp;(7)</td>
        </tr>
        <tr>
            <td>"<dfn id="notfounderror" exception export><code>NotFoundError</code></dfn>"</td>
            <td>The object can not be found here.</td>
            <td><dfn id="dom-domexception-not_found_err" for="DOMException" const export><code>NOT_FOUND_ERR</code></dfn>&nbsp;(8)</td>
        </tr>
        <tr>
            <td>"<dfn id="notsupportederror" exception export><code>NotSupportedError</code></dfn>"</td>
            <td>The operation is not supported.</td>
            <td><dfn id="dom-domexception-not_supported_err" for="DOMException" const export><code>NOT_SUPPORTED_ERR</code></dfn>&nbsp;(9)</td>
        </tr>
        <tr>
            <td>"<dfn id="inuseattributeerror" exception export><code>InUseAttributeError</code></dfn>"</td>
            <td>The attribute is in use by another [=/element=]. [[DOM]]</td>
            <td><dfn id="dom-domexception-inuse_attribute_err" for="DOMException" const export><code>INUSE_ATTRIBUTE_ERR</code></dfn>&nbsp;(10)</td>
        </tr>
        <tr>
            <td>"<dfn id="invalidstateerror" exception export><code>InvalidStateError</code></dfn>"</td>
            <td>The object is in an invalid state.</td>
            <td><dfn id="dom-domexception-invalid_state_err" for="DOMException" const export><code>INVALID_STATE_ERR</code></dfn>&nbsp;(11)</td>
        </tr>
        <tr>
            <td>"<dfn id="syntaxerror" exception export><code>SyntaxError</code></dfn>"</td>
            <td>The string did not match the expected pattern.</td>
            <td><dfn id="dom-domexception-syntax_err" for="DOMException" const export><code>SYNTAX_ERR</code></dfn>&nbsp;(12)</td>
        </tr>
        <tr>
            <td>"<dfn id="invalidmodificationerror" exception export><code>InvalidModificationError</code></dfn>"</td>
            <td>The object can not be modified in this way.</td>
            <td><dfn id="dom-domexception-invalid_modification_err" for="DOMException" const export><code>INVALID_MODIFICATION_ERR</code></dfn>&nbsp;(13)</td>
        </tr>
        <tr>
            <td>"<dfn id="namespaceerror" exception export><code>NamespaceError</code></dfn>"</td>
            <td>The operation is not allowed by <cite>Namespaces in XML</cite>. [[XML-NAMES]]</td>
            <td><dfn id="dom-domexception-namespace_err" for="DOMException" const export><code>NAMESPACE_ERR</code></dfn>&nbsp;(14)</td>
        </tr>
        <tr class="deprecated">
            <td>"<dfn id="invalidaccesserror" exception export><code>InvalidAccessError</code></dfn>"</td>
            <td>
                <strong>Deprecated.</strong>
                Use {{TypeError}} for invalid arguments,
                "{{NotSupportedError!!exception}}" {{DOMException}} for unsupported operations, and
                "{{NotAllowedError!!exception}}" {{DOMException}} for denied requests instead.
            </td>
            <td><dfn id="dom-domexception-invalid_access_err" for="DOMException" const><code>INVALID_ACCESS_ERR</code></dfn>&nbsp;(15)</td>
        </tr>
        <tr class="deprecated">
            <td>"<dfn id="typemismatcherror" exception export><code>TypeMismatchError</code></dfn>"</td>
            <td><strong>Deprecated.</strong> Use {{TypeError}} instead.</td>
            <td><dfn id="dom-domexception-type_mismatch_err" for="DOMException" const><code>TYPE_MISMATCH_ERR</code></dfn>&nbsp;(17)</td>
        </tr>
        <tr>
            <td>"<dfn id="securityerror" exception export><code>SecurityError</code></dfn>"</td>
            <td>The operation is insecure.</td>
            <td><dfn id="dom-domexception-security_err" for="DOMException" const export><code>SECURITY_ERR</code></dfn>&nbsp;(18)</td>
        </tr>
        <tr>
            <td>"<dfn id="networkerror" exception export><code>NetworkError</code></dfn>"</td>
            <td>A network error occurred.</td>
            <td><dfn id="dom-domexception-network_err" for="DOMException" const export><code>NETWORK_ERR</code></dfn>&nbsp;(19)</td>
        </tr>
        <tr>
            <td>"<dfn id="aborterror" exception export><code>AbortError</code></dfn>"</td>
            <td>The operation was aborted.</td>
            <td><dfn id="dom-domexception-abort_err" for="DOMException" const export><code>ABORT_ERR</code></dfn>&nbsp;(20)</td>
        </tr>
        <tr class="deprecated">
            <td>"<dfn id="urlmismatcherror" exception export><code>URLMismatchError</code></dfn>"</td>
            <td><strong>Deprecated.</strong></td>
            <td><dfn id="dom-domexception-url_mismatch_err" for="DOMException" const export><code>URL_MISMATCH_ERR</code></dfn>&nbsp;(21)</td>
        </tr>
        <tr class="deprecated">
            <td>"<code>QuotaExceededError</code>"</td>
            <td><strong>Deprecated.</strong> Use the {{QuotaExceededError}} {{DOMException}}-derived interface instead.</td>
            <td><dfn id="dom-domexception-quota_exceeded_err" for="DOMException" const export><code>QUOTA_EXCEEDED_ERR</code></dfn>&nbsp;(22)</td>
        </tr>
        <tr>
            <td>"<dfn id="timeouterror" exception export><code>TimeoutError</code></dfn>"</td>
            <td>The operation timed out.</td>
            <td><dfn id="dom-domexception-timeout_err" for="DOMException" const export><code>TIMEOUT_ERR</code></dfn>&nbsp;(23)</td>
        </tr>
        <tr>
            <td>"<dfn id="invalidnodetypeerror" exception export><code>InvalidNodeTypeError</code></dfn>"</td>
            <td>The supplied [=node=] is incorrect or has an incorrect ancestor for this operation. [[DOM]]</td>
            <td><dfn id="dom-domexception-invalid_node_type_err" for="DOMException" const export><code>INVALID_NODE_TYPE_ERR</code></dfn>&nbsp;(24)</td>
        </tr>
        <tr>
            <td>"<dfn id="datacloneerror" exception export><code>DataCloneError</code></dfn>"</td>
            <td>The object can not be cloned.</td>
            <td><dfn id="dom-domexception-data_clone_err" for="DOMException" const export><code>DATA_CLONE_ERR</code></dfn>&nbsp;(25)</td>
        </tr>
        <tr>
            <td>"<dfn id="encodingerror" exception export><code>EncodingError</code></dfn>"</td>
            <td>The encoding operation (either encoded or decoding) failed.</td>
            <td>—</td>
        </tr>
        <tr>
            <td>"<dfn id="notreadableerror" exception export><code>NotReadableError</code></dfn>"</td>
            <td>The I/O read operation failed.</td>
            <td>—</td>
        </tr>
        <tr>
            <td>"<dfn id="unknownerror" exception export><code>UnknownError</code></dfn>"</td>
            <td>The operation failed for an unknown transient reason (e.g. out of memory).</td>
            <td>—</td>
        </tr>
        <tr>
            <td>"<dfn id="constrainterror" exception export><code>ConstraintError</code></dfn>"</td>
            <td>A mutation operation in a transaction failed because a constraint was not satisfied. [[INDEXEDDB]]</td>
            <td>—</td>
        </tr>
        <tr>
            <td>"<dfn id="dataerror" exception export><code>DataError</code></dfn>"</td>
            <td>Provided data is inadequate.</td>
            <td>—</td>
        </tr>
        <tr>
            <td>"<dfn id="transactioninactiveerror" exception export><code>TransactionInactiveError</code></dfn>"</td>
            <td>A request was placed against a transaction which is currently not active, or which is finished. [[INDEXEDDB]]</td>
            <td>—</td>
        </tr>
        <tr>
            <td>"<dfn id="readonlyerror" exception export><code>ReadOnlyError</code></dfn>"</td>
            <td>The mutating operation was attempted in a "readonly" transaction. [[INDEXEDDB]]</td>
            <td>—</td>
        </tr>
        <tr>
            <td>"<dfn id="versionerror" exception export><code>VersionError</code></dfn>"</td>
            <td>An attempt was made to open a database using a lower version than the existing version. [[INDEXEDDB]]</td>
            <td>—</td>
        </tr>
        <tr>
            <td>"<dfn id="operationerror" exception export><code>OperationError</code></dfn>"</td>
            <td>The operation failed for an operation-specific reason.</td>
            <td>—</td>
        </tr>
        <tr>
            <td>"<dfn id="notallowederror" exception export><code>NotAllowedError</code></dfn>"</td>
            <td>The request is not allowed by the user agent or the platform in the current context, possibly because the user denied permission.</td>
            <td>—</td>
        </tr>
        <tr>
            <td>"<dfn id="optouterror" exception export><code>OptOutError</code></dfn>"</td>
            <td>The user opted out of the process.</td>
            <td>—</td>
        </tr>
    </tbody>
</table>
```
