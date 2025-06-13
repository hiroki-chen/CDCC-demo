import React, { useEffect, useState } from 'react';
import { observer } from 'mobx-react-lite';
import { Button, Card } from '@/components';
import './index.less';
import { SecureClient } from '@/biz/handlers';

async function attestationButton(client: SecureClient) {
    console.log("Attestation button clicked, client:", client);

    try {
        const response = await client.policyStyxAttestationRequst();
        console.log("✅ Attestation request sent successfully.");

        // Now check if the attestation report is genuine.
        const report = response.quote;
        if (!report) {
            throw new Error("Response does not contain 'report' field.");
        }

        // Convert back to ArrayBuffer.
        const reportBuffer = Uint8Array.from(atob(report), c => c.charCodeAt(0)).buffer;
        console.log("Converted report to ArrayBuffer, size:", reportBuffer.byteLength);

        // Now we verify the quote.
        const isValid = await client.verifyQuote(reportBuffer);
        if (!isValid) {
            throw new Error("Quote verification failed.");
        }

        console.log("✅ Quote verification successful.");

        const gy = response.gy;
        if (!gy) {
            throw new Error("Response does not contain 'gy' field.");
        }
        console.log("Received gy:", gy);

        // Convert gy from Base64 to ArrayBuffer
        const gyBuffer = Uint8Array.from(atob(gy), c => c.charCodeAt(0)).buffer;
        console.log("Converted gy to ArrayBuffer, size:", gyBuffer.byteLength);

        // Derive the shared secret using the received gy
        await client.deriveSharedSecret(gyBuffer);
        console.log("✅ Shared secret derived successfully: ", client.session_key);
    } catch (error) {
        console.error("🔥 Error during attestation request:", error);
    }
}

const HomeOne = () => {
    // 1. Store the client instance in state (created only once)
    const [client] = useState(() => new SecureClient());

    // 2. Add state to track if the keys are ready
    const [isKeyReady, setIsKeyReady] = useState(false);

    // 3. Use useEffect to run the async key generation when the component mounts
    useEffect(() => {
        // Define an async function inside the effect
        const initializeClient = async () => {
            try {
                console.log("Initializing client and generating keys...");
                await client.generateClientKeyPair();
                // 4. Once generation is complete, update the state
                setIsKeyReady(true);
            } catch (error) {
                console.error("Could not initialize secure client:", error);
                // Optionally show an error message to the user
            }
        };

        initializeClient();
    }, [client]); // The effect depends on the client instance

    return (
        <div className='home-one-root'>
            <Card>
                <h2> Initialize Attestation</h2>
                <p>
                    This page is designed to initialize the attestation process for a data owner.
                    It provides a user-friendly interface to set up and manage attestation settings.
                </p>
                {/* 5. Use the state to control the button */}
                <Button
                    onClick={() => { attestationButton(client); }}
                >
                    {isKeyReady ? 'Start Attestation' : 'Generating Keys...'}
                </Button>
            </Card>
        </div>
    );
};

export default observer(HomeOne);

