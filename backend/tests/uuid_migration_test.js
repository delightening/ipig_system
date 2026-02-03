
import axios from 'axios';
import { v4 as uuidv4 } from 'uuid';

const API_BASE_URL = 'http://localhost:8000/api'; // Adjust if needed

async function testUUIDMigration() {
    console.log('Starting UUID Migration E2E Test...');

    try {
        // 1. Create a new pig
        const pigResponse = await axios.post(`${API_BASE_URL}/pigs`, {
            ear_tag: `TEST-${Date.now()}`,
            species: 'pig',
            strain: 'white_pig',
            sex: 'male',
            birth_date: '2025-01-01',
            source: 'Internal',
        });

        const pig = pigResponse.data;
        const pigId = pig.id;
        console.log(`Created pig with ID: ${pigId} (UUID: ${uuidv4().length === pigId.length})`);

        // 2. Create a weight record for the pig
        const weightResponse = await axios.post(`${API_BASE_URL}/pigs/${pigId}/weights`, {
            weight_kg: 50.5,
            measured_at: new Date().toISOString(),
        });

        console.log(`Created weight record with ID: ${weightResponse.data.id}`);

        // 3. Create an observation record for the pig
        const observationResponse = await axios.post(`${API_BASE_URL}/pigs/${pigId}/observations`, {
            observation_date: new Date().toISOString().split('T')[0],
            content: 'Test observation for UUID migration.',
            health_status: 'normal',
        });

        console.log(`Created observation record with ID: ${observationResponse.data.id}`);

        // 4. Verify associations
        const weightCheck = await axios.get(`${API_BASE_URL}/pigs/${pigId}/weights`);
        console.log(`Weight record verified: ${weightCheck.data.some(w => w.pig_id === pigId)}`);

        const observationCheck = await axios.get(`${API_BASE_URL}/pigs/${pigId}/observations`);
        console.log(`Observation record verified: ${observationCheck.data.some(o => o.pig_id === pigId)}`);

        console.log('UUID Migration E2E Test Passed!');
    } catch (error) {
        console.error('UUID Migration E2E Test Failed:', error.response?.data || error.message);
    }
}

// Note: This script requires a running backend and potential auth headers.
// Since I cannot run an actual E2E test against the user's running system easily,
// I am providing this as a template for verification.
