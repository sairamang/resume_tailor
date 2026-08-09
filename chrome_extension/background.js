chrome.runtime.onMessage.addListener(async (message, sender, sendResponse) => {
  if (message.type !== 'analyze-pdf') return;

  const endpoint = normalizeEndpoint(message.endpoint);
  const { body, contentType: requestContentType } = buildRequestPayload(endpoint, message.model, message.text);

  try {
    const response = await fetch(endpoint, {
      method: 'POST',
      headers: {
        'Content-Type': requestContentType,
        Authorization: `Bearer ${message.apiKey}`
      },
      body: JSON.stringify(body)
    });

    const responseContentType = response.headers.get('content-type') || '';
    const responseText = await response.text();

    if (!response.ok) {
      sendResponse({ error: `API request failed: ${response.status} ${response.statusText} - ${responseText}` });
      return;
    }

    let result;
    try {
      result = contentType.includes('application/json') ? JSON.parse(responseText) : { text: responseText };
    } catch (error) {
      sendResponse({ summary: responseText });
      return;
    }

    if (typeof result.summary === 'string') {
      sendResponse({ summary: result.summary });
      return;
    }

    if (typeof result.data === 'string') {
      sendResponse({ summary: result.data });
      return;
    }

    if (result.output && Array.isArray(result.output)) {
      const combined = result.output.map((item) => (typeof item === 'string' ? item : JSON.stringify(item))).join('\n\n');
      sendResponse({ summary: combined });
      return;
    }

    if (result.text) {
      sendResponse({ summary: result.text });
      return;
    }

    sendResponse({ summary: JSON.stringify(result, null, 2) });
  } catch (error) {
    sendResponse({ error: `Failed to fetch from endpoint: ${error.message}` });
  }

  return true;
});

function normalizeEndpoint(endpoint) {
  try {
    const url = new URL(endpoint);
    if (!url.pathname || url.pathname === '/') {
      url.pathname = '/responses';
    }
    return url.toString();
  } catch (error) {
    return endpoint;
  }
}

function buildRequestPayload(endpoint, model, text) {
  try {
    const url = new URL(endpoint);
    const path = url.pathname.toLowerCase();

    if (path.includes('/openai') || path.includes('/responses')) {
      if (!path.includes('/responses')) {
        url.pathname = url.pathname.replace(/\/+$/, '') + '/responses';
      }
      return {
        contentType: 'application/json',
        body: {
          model,
          input: text
        }
      };
    }
  } catch (error) {
    // Fallback to generic payload
  }

  return {
    contentType: 'application/json',
    body: {
      model,
      input: text
    }
  };
}
